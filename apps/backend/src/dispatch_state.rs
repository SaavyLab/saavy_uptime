use worker::{D1Database, Result};

use crate::d1c::queries::monitor_dispatches::{
    finalize_dispatch as finalize_dispatch_query,
    mark_dispatch_running as mark_dispatch_running_query, upsert_dispatch_pending,
};

pub async fn record_pending_dispatch(
    d1: &D1Database,
    monitor_id: &str,
    org_id: &str,
    dispatch_id: &str,
    scheduled_for_ts: i64,
    now_ms: i64,
) -> Result<()> {
    upsert_dispatch_pending(
        d1,
        monitor_id,
        dispatch_id,
        org_id,
        scheduled_for_ts,
        now_ms,
    )
    .await
}

pub async fn mark_dispatch_running(
    d1: &D1Database,
    monitor_id: &str,
    dispatch_id: &str,
    runner_colo: Option<&str>,
    dispatched_at_ts: i64,
) -> Result<()> {
    mark_dispatch_running_query(
        d1,
        dispatched_at_ts,
        runner_colo,
        dispatched_at_ts,
        monitor_id,
        dispatch_id,
    )
    .await
}

pub async fn finalize_dispatch(
    d1: &D1Database,
    monitor_id: &str,
    dispatch_id: &str,
    status: &str,
    completed_at_ts: i64,
    error: Option<&str>,
) -> Result<()> {
    finalize_dispatch_query(
        d1,
        status,
        completed_at_ts,
        error,
        completed_at_ts,
        monitor_id,
        dispatch_id,
    )
    .await
}
