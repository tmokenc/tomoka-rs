use crate::commands::prelude::*;
use crate::traits::Embedable as _;
use crate::traits::ChannelExt as _;

#[command]
#[only_in("guilds")]
/// Check saucing status for this server
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let data = config
        .guilds
        .get(&guild_id);
        
    let mut send_embed = msg.channel_id
        .send_embed(ctx)
        .with_title("Saucing information")
        .with_color(config.color.information)
        .with_thumbnail(&config.sauce.thumbnail)
        .with_timestamp(now());
    
    if let Some(e) = &data {
        e.find_sauce.append_to(send_embed.inner_embed());
    } else {
        send_embed
            .inner_embed()
            .description("The saucing machine is disabled for this server");
    }
        
    drop(data);
    drop(config);
    send_embed.await?;
    
    Ok(())
}