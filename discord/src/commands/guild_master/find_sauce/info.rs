use crate::commands::prelude::*;
use magic::traits::MagicIter;
use crate::traits::ToEmbed;

#[command]
#[only_in("guilds")]
/// Check saucing status for this server
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
        embed.title("Saucing information");
        embed.thumbnail("https://www.daringgourmet.com/wp-content/uploads/2017/04/Sweet-Sour-Sauce-1.jpg");
        embed.color(config.color.information);
        embed.timestamp(now());
        
        if let Some(ref g) = guild_config {
            g.find_sauce.to_embed(&mut embed);
        } else {
            embed.description("The saucing machine is disabled for this server");
        }
        
        embed
    }))?;
    
    Ok(())
}