use crate::commands::prelude::*;
use crate::traits::Embedable as _;

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
        .get(&guild_id)
        .map(|v| v.find_sauce.embed_data());
        
    let thumbnail = config.sauce.thumbnail.to_owned();
    let color = config.color.information;
    
    drop(config);
        
    msg.channel_id.send_message(ctx, |m| m.embed(|mut embed| {
        if let Some(e) = data {
            embed.0 = e;
        } else {
            embed.description("The saucing machine is disabled for this server");
        }
        
        embed.title("Saucing information");
        embed.thumbnail(thumbnail);
        embed.color(color);
        embed.timestamp(now());
        
        embed
    })).await?;
    
    Ok(())
}