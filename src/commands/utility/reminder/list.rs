use crate::commands::prelude::*;
use super::DB_KEY;
use crate::types::Reminder;
use magic::traits::MagicIter as _;

#[command]
/// List all reminders
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let text = get_data::<DatabaseKey>(ctx)
        .await
        .and_then(|v| v.open(&DB_KEY).ok())
        .ok_or(magic::Error)?
        .get_all::<i64, Reminder>()
        .map(|(_, v)| v)
        .filter(|v| v.user_id == msg.author.id.0)
        .zip(1..)
        .map(|(v, i)| format!("**{}.** *{}* __{}__", i, v.when.format("%F %T UTC"), v.content.unwrap_or_default()))
        .join('\n');
    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.description(if text.is_empty() {
            "You haven't set any reminder yet"
        } else {
            text.as_str()
        });
        
        embed.footer(|f| f.text("Use `tomo>reminder remove {index}` to remove a reminder"));
        embed.timestamp(now());
        embed
    })).await?;
    
    Ok(())
}