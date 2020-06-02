use crate::commands::prelude::*;
use crate::traits::{Embedable, Paginator};
use serenity::model::channel::ReactionType;
use std::time::Duration;
use requester::NhentaiScraper;

#[command]
#[aliases("nhen")]
async fn nhentai(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !is_nsfw_channel(ctx, msg.channel_id).await {
        return Ok(())
    }
    
    let id = args.find::<u64>().ok();
    let gallery = get_data::<ReqwestClient>(ctx)
        .await
        .ok_or(magic::Error)?
        .gallery(id)
        .await?;
    
    if let Some(g) = gallery {
        let emoji = ReactionType::Unicode(String::from("📖"));
        let sent = msg.channel_id.send_message(ctx, |m| {
            m.reactions(Some(emoji.clone())).embed(|e| g.append_to(e))
        }).await?;
        let emoji_data = emoji.as_data();
        
        let collector = sent
            .await_reaction(&ctx)
            .timeout(Duration::from_secs(30))
            .filter(move |v| v.emoji.as_data().as_str() == &emoji_data)
            .removed(false)
            .await;
    
        ctx.http
            .delete_reaction(msg.channel_id.0, msg.id.0, None, &emoji)
            .await?;
            
        if collector.is_some() {
            g.pagination(ctx, msg).await?;
        }
    } else {
        return Err(format!("Cannot find any with the magic number {}", id.unwrap_or(0)).into())
    };
    

    
    Ok(())
}
