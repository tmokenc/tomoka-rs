use crate::commands::prelude::*;

#[command]
#[min_args(1)]
#[bucket = "basic"]
#[usage = "<what_to_say>"]
#[example = "I'm tmokenc's waifu"]
#[description = "Tell me to say something"]
fn say(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let to_say = args.rest();

    msg.delete(&ctx)?;
    msg.channel_id.say(&ctx.http, to_say)?;

    Ok(())
}
