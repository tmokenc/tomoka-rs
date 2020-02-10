use crate::commands::prelude::*;
use crate::storages::InforKey;

use humantime::format_duration;

#[command]
#[description = "To see how long I have been up!"]
fn uptime(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let now = Utc::now();
    let uptime = ctx
        .data
        .read()
        .get::<InforKey>()
        .unwrap()
        .uptime();

    let message = format!("I have been up for **{}**", format_duration(uptime));
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|embed| {
            embed.title("Uptime");
            embed.description(message);
            embed.timestamp(now.to_rfc3339());
            
            {
                let config = crate::read_config();
                embed.color(config.color.information);
            }
            
            embed
        })
    })?;

    Ok(())
}
