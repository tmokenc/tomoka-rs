use crate::commands::prelude::*;
use futures::future;

#[command]
#[min_args(1)]
#[bucket = "basic"]
#[usage = "<what_to_say>"]
#[example = "I'm tmokenc's waifu"]
/// Tell me to say something
async fn say(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let to_say = args.rest();
    
    let del = msg.delete(ctx);
    let say = msg.channel_id.say(&ctx.http, to_say);
    
    future::try_join(del, say).await?;

    Ok(())
}
