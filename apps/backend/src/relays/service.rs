use cuid2::create_id;
use serde::Serialize;
use worker::{wasm_bindgen::JsValue, D1Database, Method, Request, RequestInit, Stub};

use crate::cloudflare::durable_objects::location_hint::{
    DurableObjectJurisdiction, DurableObjectLocationHint,
};
use crate::cloudflare::durable_objects::relay::AppRelays;
use crate::d1c::queries::relays::{
    find_relay_by_id as find_relay_by_id_query, find_relay_by_slug as find_relay_by_slug_query,
    insert_relay as insert_relay_query, list_relays as list_relays_query, FindRelayByIdRow,
    ListRelaysRow,
};
use crate::relays::errors::RelayError;
use crate::relays::types::{RegisterRelayPayload, RelayRecord};
use crate::utils::date::now_ms;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RelayBootstrapPayload<'a> {
    relay_id: &'a str,
    slug: &'a str,
    name: &'a str,
    location_hint: &'a str,
    jurisdiction: &'a str,
}

pub async fn register_relay(
    relays: &AppRelays,
    d1: &D1Database,
    payload: RegisterRelayPayload,
) -> Result<RelayRecord, RelayError> {
    let slug = normalize_slug(&payload.slug)?;
    let name = normalize_name(&payload.name)?;
    let location_hint = payload
        .location_hint
        .parse::<DurableObjectLocationHint>()
        .map_err(|_| RelayError::validation("locationHint", "unsupported location hint"))?;
    let derived_jurisdiction = derive_jurisdiction_for_location(&location_hint);
    let jurisdiction_label = jurisdiction_label_for_location(&location_hint);

    if slug_exists(d1, &slug).await? {
        return Err(RelayError::conflict("slug"));
    }

    let now = now_ms();
    let relay_id = create_id().to_string();

    let namespace = relays.namespace();
    let object_id = match derived_jurisdiction {
        Some(ref jurisdiction) => namespace
            .unique_id_with_jurisdiction(jurisdiction.as_str())
            .map_err(|err| RelayError::durable_object("relays.unique_id", err))?,
        None => namespace
            .unique_id()
            .map_err(|err| RelayError::durable_object("relays.unique_id", err))?,
    };
    let durable_object_id = object_id.to_string();

    let stub = object_id
        .get_stub_with_location_hint(location_hint.as_str())
        .map_err(|err| RelayError::durable_object("relays.stub", err))?;

    bootstrap_relay(
        &stub,
        &RelayBootstrapPayload {
            relay_id: &relay_id,
            slug: &slug,
            name: &name,
            location_hint: location_hint.as_str(),
            jurisdiction: jurisdiction_label,
        },
    )
    .await?;

    insert_relay_query(
        d1,
        &relay_id,
        &slug,
        &name,
        location_hint.as_str(),
        jurisdiction_label,
        &durable_object_id,
        1,
        Some(now),
        now,
        now,
    )
    .await
    .map_err(|err| RelayError::database("relays.insert.exec", err))?;

    Ok(RelayRecord {
        id: relay_id,
        slug,
        name,
        location_hint: location_hint.as_str().to_string(),
        jurisdiction: jurisdiction_label.to_string(),
        durable_object_id,
        enabled: true,
        last_bootstrapped_at: Some(now),
        last_error: None,
        created_at: now,
        updated_at: now,
    })
}

fn normalize_slug(slug: &str) -> Result<String, RelayError> {
    let normalized = slug.trim().to_ascii_lowercase();
    if normalized.len() < 2 || normalized.len() > 48 {
        return Err(RelayError::validation(
            "slug",
            "slug must be between 2 and 48 characters",
        ));
    }

    if !normalized
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(RelayError::validation(
            "slug",
            "slug may only contain lowercase letters, numbers, or hyphens",
        ));
    }

    Ok(normalized)
}

fn normalize_name(name: &str) -> Result<String, RelayError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(RelayError::validation("name", "name is required"));
    }
    if trimmed.len() > 80 {
        return Err(RelayError::validation(
            "name",
            "name must be 80 characters or fewer",
        ));
    }
    Ok(trimmed.to_string())
}

fn derive_jurisdiction_for_location(
    hint: &DurableObjectLocationHint,
) -> Option<DurableObjectJurisdiction> {
    match hint {
        DurableObjectLocationHint::Weur | DurableObjectLocationHint::Eeur => {
            DurableObjectJurisdiction::new("eu").ok()
        }
        _ => None,
    }
}

fn jurisdiction_label_for_location(hint: &DurableObjectLocationHint) -> &'static str {
    match hint {
        DurableObjectLocationHint::Weur | DurableObjectLocationHint::Eeur => "eu",
        _ => "global",
    }
}

async fn slug_exists(d1: &D1Database, slug: &str) -> Result<bool, RelayError> {
    let existing = find_relay_by_slug_query(d1, slug)
        .await
        .map_err(|err| RelayError::database("relays.slug.lookup", err))?;
    Ok(existing.is_some())
}

async fn bootstrap_relay(
    stub: &Stub,
    payload: &RelayBootstrapPayload<'_>,
) -> Result<(), RelayError> {
    let body = serde_json::to_string(payload)
        .map_err(|err| RelayError::serialization("relays.bootstrap.serialize", err))?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    init.with_body(Some(JsValue::from_str(&body)));

    let mut req = Request::new_with_init("https://relay/internal/bootstrap", &init)
        .map_err(|err| RelayError::durable_object("relays.bootstrap.request", err))?;
    req.headers_mut()
        .map_err(|err| RelayError::durable_object("relays.bootstrap.headers", err))?
        .set("Content-Type", "application/json")
        .map_err(|err| RelayError::durable_object("relays.bootstrap.headers", err))?;

    stub.fetch_with_request(req)
        .await
        .map_err(|err| RelayError::durable_object("relays.bootstrap.fetch", err))?;

    Ok(())
}

fn relay_record_from_row(
    id: Option<String>,
    slug: String,
    name: String,
    location_hint: String,
    jurisdiction: String,
    durable_object_id: String,
    enabled: i64,
    last_bootstrapped_at: Option<i64>,
    last_error: Option<String>,
    created_at: i64,
    updated_at: i64,
) -> RelayRecord {
    RelayRecord {
        id: id.unwrap_or_default(),
        slug,
        name,
        location_hint,
        jurisdiction,
        durable_object_id,
        enabled: enabled != 0,
        last_bootstrapped_at,
        last_error,
        created_at,
        updated_at,
    }
}

impl From<FindRelayByIdRow> for RelayRecord {
    fn from(row: FindRelayByIdRow) -> Self {
        relay_record_from_row(
            row.id,
            row.slug,
            row.name,
            row.location_hint,
            row.jurisdiction,
            row.durable_object_id,
            row.enabled,
            row.last_bootstrapped_at,
            row.last_error,
            row.created_at,
            row.updated_at,
        )
    }
}

impl From<ListRelaysRow> for RelayRecord {
    fn from(row: ListRelaysRow) -> Self {
        relay_record_from_row(
            row.id,
            row.slug,
            row.name,
            row.location_hint,
            row.jurisdiction,
            row.durable_object_id,
            row.enabled,
            row.last_bootstrapped_at,
            row.last_error,
            row.created_at,
            row.updated_at,
        )
    }
}

pub async fn list_relays(d1: &D1Database) -> Result<Vec<RelayRecord>, RelayError> {
    let rows = list_relays_query(d1)
        .await
        .map_err(|err| RelayError::database("relays.list", err))?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get_relay_by_id(d1: &D1Database, relay_id: &str) -> Result<RelayRecord, RelayError> {
    let row = find_relay_by_id_query(d1, relay_id)
        .await
        .map_err(|err| RelayError::database("relays.find", err))?;
    row.map(Into::into)
        .ok_or_else(|| RelayError::validation("relayId", "relay not found"))
}
