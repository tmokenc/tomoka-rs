use crate::commands::prelude::*;
use crate::traits::Embedable as _;
use crate::traits::ChannelExt as _;

#[command]
#[only_in(guilds)]
/// Get the info of the repeat-words machine
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let data = config
        .guilds
        .get(&guild_id);
    
    let mut send_embed = msg.channel_id.send_embed(ctx)
        .with_title("Repeat-words information")
        .with_color(config.color.information)
        .with_timestamp(now());
    
    if let Some(e) = &data {
        e.repeat_words.append_to(send_embed.inner_embed());
    } else {
        send_embed
            .inner_embed()
            .description("The repeat-words machine doesn't running on this guild yet");
    }
        
    drop(data);
    drop(config);
    send_embed.await?;
    
    Ok(())
}