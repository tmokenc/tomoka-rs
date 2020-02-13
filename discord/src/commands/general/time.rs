use crate::commands::prelude::*;
use chrono::TimeZone;
use chrono_tz::*;
use std::env;

#[command]
#[description = "Get time for various timezone \n\
                 Passing a timestamp (from 01/01/1970 in second) to get time for a specific time \n\
                 Will support other time parsing soon:tm:"]
fn time(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
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
    
    let config = crate::read_config();
    let format = config.etc.time_format.to_owned();
    drop(config);

    let response = times
        .into_iter()
        .map(|(name, time)| {
            let tz = &time.format("%:z").to_string()[..3];
            let embed_name = format!("{} (GMT{})", name, tz);
            let embed_value = time.format(&format);
            (embed_name, embed_value.to_string(), false)
        })
        .collect::<Vec<_>>();

    msg.channel_id.send_message(ctx, |m| {
        m.embed(|embed| embed.fields(response).timestamp(msg.timestamp.to_rfc3339()))
    })?;

    Ok(())
}
