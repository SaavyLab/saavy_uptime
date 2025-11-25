use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRecord {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub location_hint: String,
    pub jurisdiction: String,
    pub durable_object_id: String,
    pub enabled: bool,
    pub last_bootstrapped_at: Option<i64>,
    pub last_error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRelayPayload {
    pub slug: String,
    pub name: String,
    pub location_hint: String,
}
