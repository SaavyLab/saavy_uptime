use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use worker::{console_error, Queue};

use crate::router::AppState;

pub fn get_queue(env: &worker::Env, name: &str) -> Result<Queue, worker::Error> {
    env.queue(name)
}

#[derive(Debug)]
pub struct TraceQueue(pub Queue);

impl FromRequestParts<AppState> for TraceQueue {
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let queue = get_queue(&state.env(), "trace-queue").map_err(|_| {
            console_error!("trace_queue.init: failed to get trace queue");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok(TraceQueue(queue))
    }
}

#[derive(Debug)]
pub struct HeartbeatQueue(pub Queue);

impl FromRequestParts<AppState> for HeartbeatQueue {
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let queue = get_queue(&state.env(), "heartbeat-queue").map_err(|_| {
            console_error!("heartbeat_queue.init: failed to get heartbeat queue");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok(HeartbeatQueue(queue))
    }
}
