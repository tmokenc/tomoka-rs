use crate::commands::prelude::*;

#[command]
#[aliases("setcachesize, resizecache, resize_cache")]
#[num_args(1)]
#[owners_only]
async fn set_cache_size(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let size = args.single::<usize>()?;

    let cache = get_data::<CacheStorage>(&ctx).await.unwrap();
    let old_size = cache.set_max_message(size).await;

    msg.channel_id.send_message(&ctx, |m| m.embed(|embed| {
        embed
            .description("The maximum number of message to be cached has been updated!!!")
            .field("Old value", old_size, true)
            .field("New value", size, true)
            .color(0x44eabe)
            .timestamp(Utc::now().to_rfc3339())
    })).await?;

    Ok(())
}
