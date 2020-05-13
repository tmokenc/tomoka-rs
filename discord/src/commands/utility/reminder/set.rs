use crate::commands::prelude::*;
use crate::types::Reminder;
use super::DB_KEY;
use humantime::{format_duration, parse_duration};
use futures::future::{self, TryFutureExt};
use std::sync::Arc;

const MAX_LIMIT_DURATION: u64 = 60 * 60 * 24 * 90;

#[command]
/// Set a reminder
async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let dm_check = msg
        .author
        .id
        .create_dm_channel(ctx)
        .map_err(|_| String::from("Cannot create DM channel to send the reminder"));
        
    let db = get_data::<DatabaseKey>(ctx)
        .await
        .and_then(|db| db.open(&DB_KEY).ok())
        .map(Arc::new)
        .ok_or(magic::Error)?;
        
    let author = msg.author.id.0;
    let db_arc = Arc::clone(&db);
        
    let db_check = tokio::task::spawn_blocking(move || {
        let count = db_arc.get_all::<i64, Reminder>()
            .filter(|(_, v)| v.user_id == author)
            .count();
            
        if count < 5 {
            Ok(())
        } else {
            Err(String::from("You currently have 5 reminders already"))
        }
    })
    .map_err(|err| err.to_string())
    .and_then(|v| async move { v });
        
    if let Err(why) = future::try_join(dm_check, db_check).await {
        msg.channel_id.say(ctx, why).await?;
        return Ok(())
    }
    
    args.trimmed();
    
    let mut duration = match args.current().and_then(|s| parse_duration(s).ok()) {
        Some(d) => {
            args.advance();
            d
        },
        None => {
            msg.channel_id.say(ctx, format!("Cannot parse the duration from `{}`", args.rest())).await?;
            return Ok(());
        }
    };
    
    while let Some(d) = args.current().and_then(|s| parse_duration(s).ok()) {
        duration += d;
        args.advance();
    }
    
    if duration.as_secs() > MAX_LIMIT_DURATION {
        msg.channel_id.say(ctx, format!("The reminder cannot be greater than 90 days")).await?;
        return Ok(())
    }
    
    let chrono_duration = match chrono::Duration::from_std(duration) {
        Ok(d) => d,
        Err(_) => {
            msg.channel_id.say(ctx, "The duration is *somewhat* invalid for me to process...").await?;
            return Ok(())
        }
    };
    
    let notify = match get_data::<ReminderNotify>(ctx).await {
        Some(s) => s,
        None => return Err("The reminder system hasn't initialized yet, please wait a few nanosecond and try again".into()),
    };
    
    let message = args.rest();
    let reminder = Reminder::new(msg, duration, &message);
    let date = reminder.when + chrono_duration;
    let timestamp = date.timestamp();
    let color = reminder.when.timestamp() as u64 & 0xffffff;
    
    tokio::task::spawn_blocking(move || {
        info!("Got a reminder for {}", &timestamp);
        db.insert(&timestamp, &reminder)
    }).await??;
    
    notify.notify();
    
    msg.channel_id.send_message(ctx, move |m| m.embed(move |embed| {
        let formated_duration = format_duration(duration);
        let formated_date = date.format("%F %T UTC");
        let mess = format!("I will remind you in **{}**", formated_duration);
        
        embed.description(mess);
        embed.title(":alarm_clock: Reminder");
        embed.image("https://cdn.discordapp.com/attachments/450521152272728065/708817978594033804/Diancie.gif");
        embed.color(color);
        embed.timestamp(now());
        
        if !message.is_empty() {
            embed.field("Message", message, false);
        }
        
        embed.field("Appointment Date", formated_date, false);
        embed
    })).await?;
    
    Ok(())
}
