use crate::router::AppState;
use worker::{D1Database, Result as WorkerResult};

pub fn get_d1(state: &AppState) -> WorkerResult<D1Database> {
    state.env().d1("DB")
}
