use std::result::Result;
use worker::{D1Database, Env};

pub fn get_d1(env: &Env) -> Result<D1Database, worker::Error> {
    env.d1("DB")
}
