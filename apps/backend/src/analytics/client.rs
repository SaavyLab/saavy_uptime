use serde::de::DeserializeOwned;
use worker::{
    console_log, wasm_bindgen::JsValue, Env, Fetch, Method, Request, RequestInit, Result,
};

#[derive(Clone)]
pub struct AeQueryClient {
    endpoint: String,
    api_token: String,
    dataset: String,
}

impl AeQueryClient {
    pub async fn from_env(env: &Env) -> Result<Self> {
        let account_id = env.var("AE_ACCOUNT_ID")?.to_string();
        let dataset = env.var("AE_HEARTBEATS_DATASET")?.to_string();
        let api_token = load_api_token(env).await?;

        let endpoint = format!(
            "https://api.cloudflare.com/client/v4/accounts/{account_id}/analytics_engine/sql"
        );

        Ok(Self {
            endpoint,
            api_token,
            dataset,
        })
    }

    pub fn dataset(&self) -> &str {
        &self.dataset
    }

    #[tracing::instrument(
        name = "analytics.client.query",
        skip(self, sql),
        fields(sql = %sql)
    )]
    pub async fn query<T: DeserializeOwned>(&self, sql: &str) -> Result<T> {
        let mut init = RequestInit::new();
        init.with_method(Method::Post);
        init.with_body(Some(JsValue::from_str(sql)));

        let mut req = Request::new_with_init(&self.endpoint, &init)?;
        {
            let headers = req.headers_mut()?;
            headers.set("Authorization", &format!("Bearer {}", self.api_token))?;
            headers.set("Content-Type", "text/plain")?;
        }

        console_log!("Analytics Engine request: {:?}", req.url());

        let mut resp = Fetch::Request(req).send().await?;
        let status = resp.status_code();
        let body = resp.text().await?;

        if status >= 400 {
            return Err(worker::Error::RustError(format!(
                "Analytics Engine query failed with status {}: {}",
                status, body
            )));
        }

        serde_json::from_str::<T>(&body).map_err(|err| {
            worker::Error::RustError(format!(
                "Failed to parse Analytics Engine response: {err:?}. Body: {body}"
            ))
        })
    }
}

async fn load_api_token(env: &Env) -> Result<String> {
    let secret = env.secret_store("AE_API_TOKEN")?.get().await?;
    match secret {
        Some(secret) => Ok(secret),
        None => Err(worker::Error::RustError(
            "Missing Analytics Engine API token. Set AE_API_TOKEN as a secret or AE_API_TOKEN_PLAINTEXT as a plain var.".
                to_string(),
        )),
    }
}
