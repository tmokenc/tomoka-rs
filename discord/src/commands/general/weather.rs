use crate::commands::prelude::*;

#[command]
#[min_args(1)]
#[usage = "<location>"]
#[example = "Akihabara"]
#[description = "Check the weather of a specific location"]
fn weather(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {

    Ok(())
}
