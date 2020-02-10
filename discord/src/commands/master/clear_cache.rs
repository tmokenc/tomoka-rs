use crate::commands::prelude::*;
use magic::report_bytes;

#[command]
#[aliases("clearcache", "cleancache")]
#[description = "Clear the __*custom cache*__ for message \
and then response with a total number of caches has been deleted and total size of files has been deleted on disk (in bytes)"]
#[owners_only]
fn clear_cache(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let cache = get_data::<CacheStorage>(&ctx).unwrap();
    let (length, size) = cache.clear()?;

    msg.channel_id.send_message(&ctx.http, |m| m.embed(|embed| {
        embed.description("Cleared the custom cache");
        embed.field("Message cached", length, true);
        embed.field("Temp files", report_bytes(size as _), true);
        
        {
            let config = crate::read_config();
            embed.color(config.color.information);
        }
        
        embed
    }))?;

    Ok(())
}
