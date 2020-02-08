use crate::commands::prelude::*;
use crate::storages::InforKey;

use humantime::format_duration;
use std::time::Duration;

#[command]
#[description = "To see how long I have been up!"]
fn uptime(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let now = Utc::now();
    let up = ctx
        .data
        .read()
        .get::<InforKey>()
        .unwrap()
        .uptime
        .timestamp_millis();
    let duration = Duration::from_millis((now.timestamp_millis() - up) as u64);

    let message = format!("I have been up for **{}**", format_duration(duration));
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Uptime")
                .description(message)
                .color(INFORMATION_COLOR)
                .timestamp(now.to_rfc3339())
        })
    })?;

    Ok(())
}
