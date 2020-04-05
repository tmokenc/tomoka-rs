use crate::commands::prelude::*;

#[command]
/// Ping me!
async fn ping(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let now = Utc::now().timestamp_millis();
    let delay = now - msg.timestamp.timestamp_millis();

    let message = format!("**Pong!** *{}ms*", delay);
    msg.channel_id.say(&ctx.http, message).await?;

    Ok(())
}
