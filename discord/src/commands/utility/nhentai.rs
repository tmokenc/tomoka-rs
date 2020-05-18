use crate::commands::prelude::*;
use crate::traits::Embedable;
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
        msg.channel_id.send_message(ctx, |m| m.embed(|e| g.append_to(e))).await?;
    } else {
        return Err(format!("Cannot find any with the magic number {}", id.unwrap_or(0)).into())
    }
    
    Ok(())
}