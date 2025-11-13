use crate::router::AppState;
use worker::{D1Database, Result};

pub fn get_d1(state: &AppState) -> Result<D1Database> {
    state.env().d1("DB")
}
