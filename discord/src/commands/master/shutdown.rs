use crate::commands::prelude::*;
use magic::ancient_magic;

#[command]
#[owners_only]
fn shutdown(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    msg.channel_id.say(ctx, "Shutting down...")?;
    ancient_magic::kill(None);
    Ok(())
}
