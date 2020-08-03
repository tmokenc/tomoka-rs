use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;

#[command]
#[bucket = "basic"]
fn decode(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    
    Ok(())
}
