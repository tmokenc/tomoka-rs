use crate::commands::prelude::*;
use chrono::TimeZone;
use chrono_tz::*;
use std::env;

#[command]
/// Get time for various timezone
/// Passing a timestamp (from 01/01/1970 in second) to get time for a specific time
/// Will support other time parsing soon:tm:
async fn time(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let time = args
        .find::<i64>()
        .unwrap_or_else(|_| msg.timestamp.timestamp());

    let utc = UTC.timestamp(time, 0);
    let times = vec![
        ("Pacific", utc.with_timezone(&US::Pacific)),
        ("UTC", utc),
        ("CET", utc.with_timezone(&CET)),
        ("Vietnam", utc.with_timezone(&Asia::Ho_Chi_Minh)),
        ("Japan", utc.with_timezone(&Japan)),
    ];
    
    let config = crate::read_config().await;
    let format = config.time.format.to_owned();
    drop(config);
    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        for (name, time) in times {
            let tz = &time.format("%:z").to_string()[..3];
            let embed_name = format!("{} (GMT{})", name, tz);
            let embed_value = time.format(&format);
            embed.field(embed_name, embed_value.to_string(), false);
        }
        
        embed.timestamp(msg.timestamp.to_rfc3339());
        embed
    })).await?;

    Ok(())
}
