use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, RwLock},
};

use crate::auth::current_user::CurrentUser;
use axum::{
    extract::FromRequestParts,
    http::{header::COOKIE, request::Parts, HeaderMap, StatusCode},
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use js_sys::Date;
use once_cell::sync::Lazy;
use rsa::{
    pkcs1v15::{Signature, VerifyingKey},
    signature::Verifier,
    RsaPublicKey,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::Sha256;
use worker::console_log;
use worker::worker_sys::console_error;
use worker::{Error, Fetch, Method, Request as WorkerRequest};

type WorkerResult<T> = worker::Result<T>;

use crate::router::AppState;
use crate::utils::date::now_s;

const JWKS_CACHE_TTL_MS: f64 = 10.0 * 60.0 * 1000.0; // 10 minutes

#[derive(Clone, Debug)]
pub struct AccessConfig {
    pub team_domain: Arc<String>,
    pub audience: Arc<String>,
}

impl AccessConfig {
    pub fn new(team_domain: String, audience: String) -> Self {
        Self {
            team_domain: Arc::new(team_domain),
            audience: Arc::new(audience),
        }
    }

    fn issuer(&self) -> String {
        self.team_domain.to_string()
    }

    pub fn team_name(&self) -> String {
        let domain = self
            .team_domain
            .strip_prefix("https://")
            .or_else(|| self.team_domain.strip_prefix("http://"))
            .unwrap_or(&self.team_domain);

        domain.split('.').next().unwrap_or(domain).to_string()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CfAccessClaims {
    pub aud: Vec<String>,
    pub email: String,
    pub exp: i64,
    pub iss: String,
    pub sub: String,
    pub name: Option<String>,
}

#[derive(Clone, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Clone, Deserialize)]
struct Jwk {
    kty: String,
    kid: String,
    n: String,
    e: String,
}

#[derive(Deserialize)]
struct JwtHeader {
    alg: String,
    kid: String,
}

#[derive(Clone)]
struct CachedKeys {
    fetched_at_ms: f64,
    keys: Vec<Jwk>,
}

static JWKS_CACHE: Lazy<RwLock<HashMap<String, CachedKeys>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[worker::send]
pub async fn verify_access_jwt(token: &str, config: AccessConfig) -> WorkerResult<CfAccessClaims> {
    let token = token.trim();
    let token = token.strip_prefix("Bearer ").unwrap_or(token);
    let (header_b64, payload_b64, signature_b64) = split_jwt(token)?;

    let header: JwtHeader = decode_segment(header_b64)?;
    if header.alg != "RS256" {
        return Err(auth_error("unsupported JWT algorithm"));
    }

    let jwk = find_jwk(&config, &header.kid).await?;
    verify_signature(header_b64, payload_b64, signature_b64, &jwk)?;

    let claims: CfAccessClaims = decode_segment(payload_b64)?;
    validate_claims(&claims, &config)?;
    Ok(claims)
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = (StatusCode, &'static str);

    fn from_request_parts<'a>(
        parts: &'a mut Parts,
        state: &AppState,
    ) -> impl Future<Output = std::result::Result<Self, Self::Rejection>> + Send {
        async move {
            if let Some(existing) = parts.extensions.get::<CurrentUser>() {
                return Ok(existing.clone());
            }

            let token = {
                extract_token(&parts.headers)
                    .or_else(|| extract_token_from_cookies(&parts.headers))
                    .ok_or((StatusCode::UNAUTHORIZED, "missing access token"))?
            };

            let access_config = state.access_config().clone();

            let claims = verify_access_jwt(&token, access_config)
                .await
                .map_err(|err| {
                    console_error!("{err}");
                    (StatusCode::UNAUTHORIZED, "invalid or expired token")
                })?;

            let user = CurrentUser::new(claims);
            parts.extensions.insert(user.clone());
            Ok(user)
        }
    }
}

fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("CF_Authorization")
        .or_else(|| headers.get("Cf-Access-Jwt-Assertion"))
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

fn extract_token_from_cookies(headers: &HeaderMap) -> Option<String> {
    headers
        .get(COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookie_header| {
            cookie_header
                .split(';')
                .map(|kv| kv.trim())
                .find_map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    match (parts.next(), parts.next()) {
                        (Some("CF_Authorization"), Some(token)) => Some(token.to_string()),
                        _ => None,
                    }
                })
        })
}

fn validate_claims(claims: &CfAccessClaims, config: &AccessConfig) -> WorkerResult<()> {
    let aud_match = claims.aud.iter().any(|aud| aud == &*config.audience);
    if !aud_match {
        return Err(auth_error("audience mismatch"));
    }

    if claims.iss != config.issuer() {
        return Err(auth_error("issuer mismatch"));
    }

    let now = now_s();
    if claims.exp <= now {
        return Err(auth_error("token expired"));
    }

    Ok(())
}

fn verify_signature(
    header_b64: &str,
    payload_b64: &str,
    signature_b64: &str,
    jwk: &Jwk,
) -> WorkerResult<()> {
    let signing_input = format!("{header_b64}.{payload_b64}");
    let signature_bytes = decode_segment_raw(signature_b64)?;
    let signature = Signature::try_from(signature_bytes.as_slice())
        .map_err(|_| auth_error("invalid signature bytes"))?;

    let key = jwk_to_rsa(jwk)?;
    let verifying_key = VerifyingKey::<Sha256>::new(key);
    verifying_key
        .verify(signing_input.as_bytes(), &signature)
        .map_err(|_| auth_error("JWT signature verification failed"))?;
    Ok(())
}

#[worker::send]
async fn find_jwk(config: &AccessConfig, kid: &str) -> WorkerResult<Jwk> {
    let keys = load_jwks(config).await?;
    keys.into_iter()
        .find(|key| key.kid == kid)
        .ok_or_else(|| auth_error("kid not found in JWKS"))
}

#[worker::send]
async fn load_jwks(config: &AccessConfig) -> WorkerResult<Vec<Jwk>> {
    {
        let cache = JWKS_CACHE
            .read()
            .map_err(|_| auth_error("failed to read JWKS cache"))?;
        if let Some(entry) = cache.get(config.team_domain.as_ref()) {
            if Date::now() - entry.fetched_at_ms <= JWKS_CACHE_TTL_MS {
                return Ok(entry.keys.clone());
            }
        }
    }

    let url = format!("{}/cdn-cgi/access/certs", config.team_domain.as_ref());
    let request = WorkerRequest::new(&url, Method::Get)?;
    let mut resp = Fetch::Request(request).send().await?;
    let status = resp.status_code();
    if !(200..=299).contains(&status) {
        return Err(auth_error(format!(
            "unable to fetch Access JWKS (status {status})"
        )));
    }
    let body = resp.text().await?;
    let jwks: Jwks =
        serde_json::from_str(&body).map_err(|err| auth_error(format!("invalid JWKS: {err}")))?;

    {
        let mut cache = JWKS_CACHE
            .write()
            .map_err(|_| auth_error("failed to write JWKS cache"))?;
        cache.insert(
            config.team_domain.as_ref().clone(),
            CachedKeys {
                fetched_at_ms: Date::now(),
                keys: jwks.keys.clone(),
            },
        );
    }

    Ok(jwks.keys)
}

fn jwk_to_rsa(jwk: &Jwk) -> WorkerResult<RsaPublicKey> {
    if jwk.kty != "RSA" {
        return Err(auth_error("unexpected JWK kty"));
    }

    let modulus = decode_segment_raw(&jwk.n)?;
    let exponent = decode_segment_raw(&jwk.e)?;

    let n = rsa::BigUint::from_bytes_be(&modulus);
    let e = rsa::BigUint::from_bytes_be(&exponent);
    RsaPublicKey::new(n, e).map_err(|err| auth_error(format!("invalid JWK: {err}")))
}

fn split_jwt(token: &str) -> WorkerResult<(&str, &str, &str)> {
    let mut segments = token.split('.');
    match (
        segments.next(),
        segments.next(),
        segments.next(),
        segments.next(),
    ) {
        (Some(h), Some(p), Some(s), None) => Ok((h, p, s)),
        _ => Err(auth_error("malformed JWT")),
    }
}

fn decode_segment<T>(segment: &str) -> WorkerResult<T>
where
    T: DeserializeOwned,
{
    let bytes = decode_segment_raw(segment)?;
    serde_json::from_slice(&bytes).map_err(|err| auth_error(format!("invalid JSON: {err}")))
}

fn decode_segment_raw(segment: &str) -> WorkerResult<Vec<u8>> {
    URL_SAFE_NO_PAD
        .decode(segment.as_bytes())
        .map_err(|_| auth_error("invalid base64 segment"))
}

fn auth_error<T: Into<String>>(message: T) -> Error {
    Error::RustError(format!("auth: {}", message.into()))
}
