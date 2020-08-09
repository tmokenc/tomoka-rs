use crate::commands::prelude::*;
use crate::traits::Embedable as _;
use crate::traits::ChannelExt as _;

#[command]
#[only_in("guilds")]
/// Check SadKaede-finder status for this server
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
        .with_title("SadKaede-finder information")
        .with_color(config.color.information)
        .with_timestamp(now())
        .with_thumbnail(&config.sadkaede.thumbnail);
        
    if let Some(e) = &data {
        e.find_sadkaede.append_to(send_embed.inner_embed());
    } else {
        send_embed
            .inner_embed()
            .description("The SadKaede-finder is disabled for this server");
    }
        
    drop(data);
    drop(config);
    
    send_embed.await?;
    
    Ok(())
}