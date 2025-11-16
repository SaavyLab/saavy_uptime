use std::result::Result;
use worker::{Env, Queue};

pub fn get_heartbeat_summaries_queue(env: &Env) -> Result<Queue, worker::Error> {
    env.queue("heartbeat_summaries")
}
