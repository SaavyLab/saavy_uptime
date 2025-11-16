use crate::cloudflare::queues::heartbeat_summary_types::HeartbeatSummary;
use worker::*;

#[event(queue)]
pub async fn main(
    message_batch: MessageBatch<HeartbeatSummary>,
    _env: Env,
    _ctx: Context,
) -> Result<()> {
    let messages = message_batch.messages()?;
    for message in messages {
        console_log!(
            "got messages {:?}, with id {} and ts: {}",
            message.body(),
            message.id(),
            message.timestamp().to_string()
        );

        message.ack();
    }

    Ok(())
}
