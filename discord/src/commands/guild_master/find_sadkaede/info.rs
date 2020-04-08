use crate::commands::prelude::*;
use crate::traits::Embedable as _;

#[command]
#[only_in("guilds")]
/// Check SadKaede-finder status for this server
async fn info(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let data = config
        .guilds
        .get(&guild_id)
        .map(|g| g.find_sadkaede.embed_data());
        
    let thumbnail = config.sadkaede.thumbnail.to_owned();
    let color = config.color.information;
        
    drop(config);
        
    msg.channel_id.send_message(ctx, |m| m.embed(|mut embed| {
        if let Some(g) = data {
            embed.0 = g;
        } else {
            embed.description("The SadKaede-finder is disabled for this server");
        }
        
        embed.title("SadKaede-finder information");
        embed.thumbnail(thumbnail);
        embed.color(color);
        embed.timestamp(now());
        
        embed
    })).await?;
    
    Ok(())
}