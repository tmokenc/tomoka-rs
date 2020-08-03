use crate::commands::prelude::*;
use crate::storages::InforKey;

use humantime::format_duration;

#[command]
/// To see how long I have been up!
async fn uptime(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let now = Utc::now();
    let uptime = ctx
        .data
        .read()
        .await
        .get::<InforKey>()
        .unwrap()
        .uptime();
        
    let config = crate::read_config().await;
    let color = config.color.information;
    drop(config);

    let message = format!("I have been up for **{}**", format_duration(uptime));
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|embed| {
            embed.title("Uptime");
            embed.description(message);
            embed.timestamp(now.to_rfc3339());
            
            embed.color(color);
            
            embed
        })
    }).await?;

    Ok(())
}
