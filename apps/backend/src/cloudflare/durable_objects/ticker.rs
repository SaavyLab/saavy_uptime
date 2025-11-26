use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use worker::{console_error, console_warn, Env, ObjectNamespace, Stub};

use crate::cloudflare::durable_objects::location_hint::DurableObjectLocationHint;
use crate::router::AppState;

pub fn get_ticker_do(env: &Env) -> std::result::Result<ObjectNamespace, worker::Error> {
    env.durable_object("TICKER")
}

#[derive(Debug, Clone)]
pub struct AppTicker {
    namespace: ObjectNamespace,
    default_location_hint: Option<DurableObjectLocationHint>,
}

impl AppTicker {
    pub fn new(
        namespace: ObjectNamespace,
        default_location_hint: Option<DurableObjectLocationHint>,
    ) -> Self {
        Self {
            namespace,
            default_location_hint,
        }
    }

    pub fn namespace(&self) -> &ObjectNamespace {
        &self.namespace
    }

    pub fn default_location_hint(&self) -> Option<DurableObjectLocationHint> {
        self.default_location_hint
    }

    pub fn stub_for_org(
        &self,
        org_id: &str,
        override_hint: Option<DurableObjectLocationHint>,
    ) -> std::result::Result<Stub, worker::Error> {
        let hint = override_hint.or(self.default_location_hint);
        match hint {
            Some(location_hint) => self
                .namespace
                .get_by_name_with_location_hint(org_id, location_hint.as_str()),
            None => self.namespace.get_by_name(org_id),
        }
    }

    pub fn stub(&self, org_id: &str) -> std::result::Result<Stub, worker::Error> {
        self.stub_for_org(org_id, None)
    }
}

impl FromRequestParts<AppState> for AppTicker {
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        let namespace = get_ticker_do(&state.env()).map_err(|_| {
            console_error!("ticker.do.init: failed to get ticker durable object");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let default_location_hint =
            state
                .env()
                .var("TICKER_LOCATION_HINT")
                .ok()
                .and_then(|value| {
                    let raw = value.to_string();
                    match raw.trim().parse::<DurableObjectLocationHint>() {
                        Ok(hint) => Some(hint),
                        Err(err) => {
                            if !raw.trim().is_empty() {
                                console_warn!(
                                    "ticker.do.location_hint: ignoring unsupported hint '{}': {}",
                                    raw.trim(),
                                    err
                                );
                            }
                            None
                        }
                    }
                });

        Ok(AppTicker::new(namespace, default_location_hint))
    }
}
