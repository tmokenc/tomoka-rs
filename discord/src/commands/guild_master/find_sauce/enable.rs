#![allow(unstable_name_collisions)]

use crate::commands::prelude::*;
use magic::traits::MagicIter as _;
use magic::traits::MagicBool as _;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Enable the saucing machine
async fn enable(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let (description, field) = match config.guilds.get_mut(&guild_id) {
        Some(ref mut g) if !g.find_sauce.channels.is_empty() || g.find_sauce.all => {
            let description = if g.find_sauce.enable {
                "The saucing machine already enabled"
            } else {
                g.enable_find_sauce();
                update_guild_config(&ctx, &g).await?;
                "Enabled the saucing machine"
            };
            
            let channels = g
                .find_sauce
                .channels
                .iter()
                .map(|v| format!("<#{}>", v))
                .join(" ");
            
            (description, (!channels.is_empty()).then_some(channels))
        }
            
        _ => {
            let mess = "There isn't any activated channel for sauce yet.
                Consider adding the channel(s) using `option sauce add <channels>` command.
                The add command will automatically **enable** this saucing machine";
                
            (mess, None)
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
            embed.field("Activiting channels", value, true);
        }
       
        embed
    })).await?;
    
    Ok(())
}