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
    
    let guild_config = crate::read_config()
        .guilds
        .get(&guild_id);
        
    msg.channel_id.send_message(ctx, |m| m.embed(|mut embed| {
        embed.title("Saucing information");
        embed.thumbnail("https://www.daringgourmet.com/wp-content/uploads/2017/04/Sweet-Sour-Sauce-1.jpg");
        embed.color(INFORMATION_COLOR);
        embed.timestamp(now());
        
        if let Some(ref g) = guild_config {
            g.find_sauce.to_embed(&mut embed);
        } else {
            embed.description("The saucing machine is disabled for this server");
        }
    
        // match guild_config {
        //     Some(ref g) if !g.find_sauce.enable => {
        //         embed.description("The saucing service is disabled for this server");
        //     }
        //     
        //     Some(ref g) if g.find_sauce.all => {
        //         embed.description("The saucing service is enabled for all channels on this server");
        //     }
        //     
        //     Some(ref g) if !g.find_sauce.channels.is_empty() => {
        //         let channels = &g.find_sauce.channels;
        //         let mess = format!("The saucing service is enabled for {} channels on this server", channels.len());
        //         let s = channels
        //             .iter()
        //             .map(|v| format!("<#{}>", v.0))
        //             .join(" ");
        //             
        //         embed.description(mess);
        //         embed.field("Saucing channels", s, true);
        //     }
        //     
        //     _ => {
        //         embed.description("The saucing service is disabled for this server");
        //     }
        // }
        
        embed
    }))?;
    
    Ok(())
}