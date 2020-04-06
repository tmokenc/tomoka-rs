use crate::commands::prelude::*;
use magic::traits::MagicIter as _;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Toggle the saucing machine
async fn toggle(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let (description, field) = match config.guilds.get_mut(&guild_id) {
        Some(ref mut g) if !g.find_sauce.channels.is_empty() || g.find_sauce.all => {
            g.find_sauce.enable = !g.find_sauce.enable;
            update_guild_config(&ctx, &g).await?;
            
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
                    
                    (mess, Some(channels))
                }
                
                (_, true, true) => {
                    ("Enabled the sauching machine for **ALL** channel".to_string(), None)
                }
                
                _ => ("Disabled the saucing machine...".to_string(), None)
            }
        }
        
        _ => {
            let mess = "The saucing machine isn't configurated yet.
            Consider use `option sauce add` to add channel for the sauching machine to work with";
            
            (mess.to_string(), None)
        }
    };
        
    let thumbnail = config.sauce.thumbnail.to_owned();
    let color = config.color.information;
    
    drop(config);
    
    msg.channel_id.send_message(&ctx, |m| m.embed(|embed| {
        embed.title("Saucing information");
        embed.thumbnail(thumbnail);
        embed.color(color);
        embed.timestamp(now());
        
        embed.description(description);
        
        if let Some(value) = field {
            embed.field("Activating channels", value, true);   
        }
       
        embed
    })).await?;
    
    Ok(())
}