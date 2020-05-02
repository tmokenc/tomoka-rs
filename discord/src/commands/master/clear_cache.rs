use crate::commands::prelude::*;
use magic::report_bytes;

#[command]
#[aliases("clearcache", "cleancache")]
#[owners_only]
/// Clear the __*custom cache*__ for message
/// and then response with a total number of caches has been deleted and total size of files has been deleted on disk (in bytes)
async fn clear_cache(ctx: &Context, msg: &Message) -> CommandResult {
    let cache = get_data::<CacheStorage>(&ctx).await.unwrap();
    let (length, size) = cache.clear().await?;
    
    let color = {
        let config = crate::read_config().await;
        config.color.information
    };

    msg.channel_id.send_message(&ctx.http, move |m| m.embed(|embed| {
        embed.description("Cleared the custom cache");
        embed.field("Message cached", length, true);
        embed.field("Temp files", report_bytes(size as _), true);
        embed.color(color);
        
        embed
    })).await?;

    Ok(())
}
