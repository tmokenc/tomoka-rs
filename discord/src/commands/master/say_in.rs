use crate::commands::prelude::*;
use magic::traits::MagicStr;
use serenity::model::id::ChannelId;

#[command]
#[owners_only]
/// Say in specific channel, even in other guild.
fn say_in(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel_id = match args.single::<u64>() {
        Ok(id) => ChannelId(id),
        Err(_) => {
            msg.channel_id
                .say(ctx, "Please give in a channel id (not mention)")?;
            return Ok(());
        }
    };

    let message = match args.rest().to_option() {
        Some(s) => s,
        None => {
            msg.channel_id.say(ctx, "please put in the content...")?;
            return Ok(());
        }
    };

    channel_id.broadcast_typing(&ctx)?;
    msg.channel_id.broadcast_typing(&ctx)?;

    channel_id.say(&ctx, &message)?;
    msg.channel_id.send_message(ctx, |m| {
        m.embed(|embed| {
            embed.description(format!("Sent message to channel with id {}", channel_id));
            embed.field("Sent message", message, false);
            embed
        })
    })?;

    Ok(())
}
