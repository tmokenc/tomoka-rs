use crate::commands::prelude::*;
use magic::traits::MagicIter;
use crate::traits::ToEmbed;

#[command]
#[only_in("guilds")]
/// Check SadKaede-finder status for this server
fn info(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(())
    };
    
    let config = crate::read_config();
    let guild_config = config
        .guilds
        .get(&guild_id);
        
    msg.channel_id.send_message(ctx, |m| m.embed(|mut embed| {
        embed.title("SadKaede-finder information");
        embed.thumbnail(&config.sadkaede.thumbnail);
        embed.color(config.color.information);
        embed.timestamp(now());
        
        if let Some(ref g) = guild_config {
            g.find_sadkaede.to_embed(&mut embed);
        } else {
            embed.description("The SadKaede-finder is disabled for this server");
        }
        
        embed
    }))?;
    
    Ok(())
}