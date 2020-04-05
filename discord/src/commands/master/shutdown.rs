use crate::commands::prelude::*;
use magic::ancient_magic;

#[command]
#[owners_only]
async fn shutdown(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    msg.channel_id.say(ctx, "Shutting down...").await?;
    ancient_magic::kill(None);
    Ok(())
}
