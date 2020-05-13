use super::DB_KEY;
use crate::commands::prelude::*;
use crate::types::Reminder;

#[command]
#[min_args(1)]
async fn remove(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = args.current().unwrap();
    
    let notify = match get_data::<ReminderNotify>(ctx).await {
        Some(s) => s,
        None => return Err("The reminder system hasn't initialized yet, please wait a few nanosecond and try again".to_string().into()),
    };
    
    let db = get_data::<DatabaseKey>(ctx)
        .await
        .and_then(|v| v.open(&DB_KEY).ok())
        .ok_or(magic::Error)?;
    
    let author = msg.author.id.0;
    
    if data.to_lowercase().as_str() == "all" {
        tokio::task::spawn_blocking(move || {
            let keys = db
                .get_all::<i64, Reminder>()
                .filter(|(_, v)| v.user_id == author)
                .map(|(k, _)| k);
            
            db.remove_many(keys)
        }).await??;
        
        notify.notify();
        
        msg.channel_id.say(ctx, "Removed all the reminders").await?;
        return Ok(())
    }
    
    let index = match data.parse::<usize>() {
        Ok(i) => i, 
        Err(_) => return Ok(())
    };
    
    let res = tokio::task::spawn_blocking(move || {
        let (key, val) = db
            .get_all::<i64, Reminder>()
            .filter(|(_, v)| v.user_id == author)
            .nth(index)?;
            
        db.remove(&key).ok()?;
        Some(val)
    }).await?;
    
    match res {
        Some(r) => {
            let mess = format!("Removed the reminder on **{}**", r.when.format("%F %T UTC"));
            
            notify.notify();
            msg.channel_id.say(ctx, mess).await?
        }
                
        None => msg.channel_id.say(ctx, "404 Not Found any reminder...").await?
    };
    
    Ok(())
}