use crate::commands::prelude::*;

#[command]
#[only_in(guilds)]
async fn count(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(()),
    };
    
    let len = crate::read_config()
        .await
        .guilds
        .get(&guild_id)
        .and_then(|v| v.rgblized.as_ref().map(|x| x.len()));

    let mess = if let Some(l) = len {
        format!("RGB has {} roles in total", l)
    } else {
        "Lucky! This guild hasn't been rgblized yet!".to_owned()
    };
    
    msg.channel_id.say(&ctx, mess).await?;
    Ok(())
}
