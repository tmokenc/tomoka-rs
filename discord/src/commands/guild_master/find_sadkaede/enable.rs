use crate::commands::prelude::*;
use magic::traits::MagicIter as _;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Enable the SadKaede-finder
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
        embed.title("SadKaede-finder information");
        embed.thumbnail(&config.sadkaede.thumbnail);
        embed.color(config.color.information);
        embed.timestamp(now());
        
        match guild_config {
            Some(ref mut g) if !g.find_sadkaede.channels.is_empty() || g.find_sadkaede.all => {
                if g.find_sadkaede.enable {
                    embed.description("The SadKaede-finder already enabled");
                } else {
                    g.enable_find_sadkaede();
                    update_guild_config(&ctx, &g).ok();
                    embed.description("Enabled the SadKaede-finder");
                }
                
                if !g.find_sadkaede.channels.is_empty() {
                    let channels = g
                        .find_sadkaede
                        .channels
                        .iter()
                        .map(|v| format!("<#{}>", v))
                        .join(" ");
                        
                    embed.field("Activiting channels", channels, true);
                }
                
            }
            
            _ => {
                embed.description(
                    "There isn't any activated channel for sadkaede yet.
                    Consider adding the channel(s) using `option sadkaede add <channels>` command.
                    The add command will automatically **enable** this SadKaede-finder"
                );
            }
        };
        
        embed
    }))?;
    
    Ok(())
}