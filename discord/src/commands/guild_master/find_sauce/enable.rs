use crate::commands::prelude::*;
use magic::traits::MagicIter as _;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Enable the saucing machine
fn enable(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config();
    let mut guild_config = config
        .guilds
        .get_mut(&guild_id);
        
    msg.channel_id.send_message(&ctx, |m| m.embed(|embed| {
        embed.title("Saucing information");
        embed.thumbnail(&config.sauce.thumbnail);
        embed.color(config.color.information);
        embed.timestamp(now());
        
        match guild_config {
            Some(ref mut g) if !g.find_sauce.channels.is_empty() || g.find_sauce.all => {
                if g.find_sauce.enable {
                    embed.description("The saucing machine already enabled");
                } else {
                    g.enable_find_sauce();
                    update_guild_config(&ctx, &g).ok();
                    embed.description("Enabled the saucing machine");
                }
                
                if !g.find_sauce.channels.is_empty() {
                    let channels = g
                        .find_sauce
                        .channels
                        .iter()
                        .map(|v| format!("<#{}>", v))
                        .join(" ");
                        
                    embed.field("Activiting channels", channels, true);
                }
                
            }
            
            _ => {
                embed.description(
                    "There isn't any activated channel for sauce yet.
                    Consider adding the channel(s) using `option sauce add <channels>` command.
                    The add command will automatically **enable** this saucing machine"
                );
            }
        };
        
        embed
    }))?;
    
    Ok(())
}