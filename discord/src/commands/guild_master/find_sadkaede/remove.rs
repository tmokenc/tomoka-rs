use crate::commands::prelude::*;
use magic::traits::MagicIter as _;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Remove channel(s) from the SadKaede-finder list on this server
fn remove(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(())
    };
    
    let channels = extract_channel_ids(&msg.content);
    
    if channels.is_empty() {
        msg.channel_id.send_message(ctx, |m| {
                m.content("Please *mention* some channel to be watched")
            })?;
            return Ok(());
    }
    
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
            Some(ref mut g) => {
                let removed_channels = channels
                    .into_iter()
                    .filter_map(|v| g.remove_sadkaede_channel(v))
                    .collect::<Vec<_>>();
                
                if removed_channels.is_empty() {
                    embed.description("These channels don't exist in the list...");
                } else {
                    update_guild_config(&ctx, &g).ok();
                    let mess = format!("Removed {} channels", removed_channels.len());
                    let s = removed_channels
                        .into_iter()
                        .map(|v| format!("<#{}>", v.0))
                        .join(" ");
                        
                    embed.description(mess);
                    embed.field("Channels", s, true);
                }
            }
            
            _ => {
                embed.description("These channels don't exist in the list...");
            }
        }
        
        embed
    }))?;
    
    Ok(())
}