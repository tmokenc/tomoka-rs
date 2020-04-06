use crate::commands::prelude::*;
use magic::traits::MagicIter as _;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Toggle the SadKaede-finder
async fn toggle(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let mut guild_config = config
        .guilds
        .get_mut(&guild_id);
        
    let (description, field) = match guild_config {
        Some(ref mut g) if !g.find_sadkaede.channels.is_empty() || g.find_sadkaede.all => {
            g.find_sadkaede.enable = !g.find_sadkaede.enable;
            update_guild_config(&ctx, &g).await?;
            
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
                    
                    (mess, Some(channels))
                }
                
                (_, true, true) => {
                    ("Enabled the SadKaede-finder for **ALL** channel".to_owned(), None)
                }
                
                _ => {
                    ("Disabled the SadKaede-finder...".to_owned(), None)
                }
            }
        }
        
        _ => {
            let des = "The SadKaede-finder isn't configurated yet.
            Consider use `option sadkaede add` to add channel for the SadKaede-finder to work with";
            (des.to_owned(), None)
        }
    };
        
    let thumbnail = config.sadkaede.thumbnail.to_owned();
    let color = config.color.information;
        
    drop(guild_config);
    drop(config);
        
    msg.channel_id.send_message(&ctx, |m| m.embed(|embed| {
        embed.title("SadKaede-finder information");
        embed.thumbnail(thumbnail);
        embed.color(color);
        embed.timestamp(now());
        
        embed.description(description);
        
        if let Some(f) = field {
            embed.field("Activating channels", f, true);   
        }
        
        embed
    })).await?;
    
    Ok(())
}