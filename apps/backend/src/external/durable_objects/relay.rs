use serde::{Deserialize, Serialize};
use worker::*;

use crate::utils::date::now_ms;

#[derive(Default, Serialize, Deserialize)]
struct RelayState {
    relay_id: Option<String>,
    slug: Option<String>,
    name: Option<String>,
    location_hint: Option<String>,
    jurisdiction: Option<String>,
    bootstrapped_at: Option<i64>,
    updated_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelayBootstrapPayload {
    relay_id: String,
    slug: String,
    name: String,
    location_hint: String,
    jurisdiction: String,
}

#[durable_object]
pub struct Relay {
    state: State,
    _env: Env,
}

impl Relay {
    async fn load_state(&self) -> Result<RelayState> {
        Ok(self
            .state
            .storage()
            .get::<RelayState>("state")
            .await?
            .unwrap_or_default())
    }

    async fn save_state(&self, state: &RelayState) -> Result<()> {
        self.state.storage().put("state", state).await
    }

    async fn bootstrap(&self, payload: RelayBootstrapPayload) -> Result<Response> {
        let mut state = self.load_state().await?;
        let now = now_ms();

        state.relay_id = Some(payload.relay_id);
        state.slug = Some(payload.slug);
        state.name = Some(payload.name);
        state.location_hint = Some(payload.location_hint);
        state.jurisdiction = Some(payload.jurisdiction);
        state.bootstrapped_at = Some(now);
        state.updated_at = Some(now);

        self.save_state(&state).await?;

        Response::ok("ok")
    }
}

impl DurableObject for Relay {
    fn new(state: State, env: Env) -> Self {
        Self { state, _env: env }
    }

    async fn fetch(&self, mut req: Request) -> Result<Response> {
        match (req.method(), req.path().as_str()) {
            (Method::Post, "/internal/bootstrap") => {
                let payload: RelayBootstrapPayload = req.json().await?;
                self.bootstrap(payload).await
            }
            (Method::Get, "/internal/status") => {
                let state = self.load_state().await?;
                Response::from_json(&state)
            }
            _ => Response::error("Not found", 404),
        }
    }
}
