use crate::commands::prelude::*;

#[command]
#[only_in(guilds)]
fn count(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let len = crate::read_config()
        .guilds
        .get(&guild_id)
        .and_then(|v| v.rgblized.as_ref().map(|x| x.len()));

    if let Some(l) = len {
        msg.channel_id
            .say(&ctx, format!("RGB has {} roles in total", l))?;
    } else {
        msg.channel_id
            .say(&ctx, "Lucky! This guild hasn't been rgblized yet!")?;
    }

    Ok(())
}
