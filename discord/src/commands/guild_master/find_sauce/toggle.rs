use crate::commands::prelude::*;
use magic::traits::MagicIter as _;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Toggle the saucing machine
fn toggle(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
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
                g.find_sauce.enable = !g.find_sauce.enable;
                update_guild_config(&ctx, &g).ok();
                
                let channels = &g.find_sauce.channels;
                
                match (channels.len(), g.find_sauce.all, g.find_sauce.enable) {
                    (n, false, true) if n > 0 => {
                        let channels = &g.find_sauce.channels;
                        let mess = format!("Enabled the saucing machine for {} channels", channels.len());
                        
                        let channels = g
                            .find_sauce
                            .channels
                            .iter()
                            .map(|v| format!("<#{}>", v))
                            .join(" ");
                        
                        embed.description(mess);
                        embed.field("Activating channels", channels, true);   
                    }
                    
                    (_, true, true) => {
                        embed.description("Enabled the sauching machine for **ALL** channel");
                    }
                    
                    _ => {
                        embed.description("Disabled the saucing machine...");   
                    }
                }
            }
            
            _ => {
                embed.description("The saucing machine isn't configurated yet.
                Consider use `option sauce add` to add channel for the sauching machine to work with");
            }
        }
        
        embed
    }))?;
    
    Ok(())
}