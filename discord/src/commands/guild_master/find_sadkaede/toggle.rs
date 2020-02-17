use crate::commands::prelude::*;
use magic::traits::MagicIter as _;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Toggle the SadKaede-finder
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
        embed.title("SadKaede-finder information");
        embed.thumbnail(&config.sadkaede.thumbnail);
        embed.color(config.color.information);
        embed.timestamp(now());
        
        match guild_config {
            Some(ref mut g) if !g.find_sadkaede.channels.is_empty() || g.find_sadkaede.all => {
                g.find_sadkaede.enable = !g.find_sadkaede.enable;
                update_guild_config(&ctx, &g).ok();
                
                let channels = &g.find_sadkaede.channels;
                
                match (channels.len(), g.find_sadkaede.all, g.find_sadkaede.enable) {
                    (n, false, true) if n > 0 => {
                        let channels = &g.find_sadkaede.channels;
                        let mess = format!("Enabled the SadKaede-finder for {} channels", channels.len());
                        
                        let channels = g
                            .find_sadkaede
                            .channels
                            .iter()
                            .map(|v| format!("<#{}>", v))
                            .join(" ");
                        
                        embed.description(mess);
                        embed.field("Activating channels", channels, true);   
                    }
                    
                    (_, true, true) => {
                        embed.description("Enabled the SadKaede-finder for **ALL** channel");
                    }
                    
                    _ => {
                        embed.description("Disabled the SadKaede-finder...");   
                    }
                }
            }
            
            _ => {
                embed.description("The SadKaede-finder isn't configurated yet.
                Consider use `option sadkaede add` to add channel for the SadKaede-finder to work with");
            }
        }
        
        embed
    }))?;
    
    Ok(())
}