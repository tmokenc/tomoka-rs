#![allow(unstable_name_collisions)]

use crate::commands::prelude::*;
use magic::traits::MagicIter as _;
use magic::traits::MagicBool as _;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Enable the SadKaede-finder
async fn enable(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let (description, field) = match config.guilds.get_mut(&guild_id) {
        Some(ref mut g) if !g.find_sadkaede.channels.is_empty() || g.find_sadkaede.all => {
            let description = if g.find_sadkaede.enable {
                "The SadKaede-finder already enabled"
            } else {
                g.enable_find_sadkaede();
                update_guild_config(&ctx, &g).await?;
                "Enabled the SadKaede-finder"
            };
            
            let field = (!g.find_sadkaede.channels.is_empty()).then(move || {
                g.find_sadkaede
                    .channels
                    .iter()
                    .map(|v| format!("<#{}>", v))
                    .join(" ")
            });
            
            (description, field)
        }
        
        _ => {
            let mess = "There isn't any activated channel for sadkaede yet.
                Consider adding the channel(s) using `option sadkaede add <channels>` command.
                The add command will automatically **enable** this SadKaede-finder";
            (mess, None)
        }
    };
        
    let thumbnail = config.sadkaede.thumbnail.to_owned();
    let color = config.color.information;
    
    drop(config);
        
    msg.channel_id.send_message(&ctx, |m| m.embed(|embed| {
        embed.title("SadKaede-finder information");
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