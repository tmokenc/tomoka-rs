use crate::commands::prelude::*;
use magic::traits::MagicIter as _;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Remove channel(s) from the SadKaede-finder list on this server
async fn remove(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(())
    };
    
    let channels = extract_channel_ids(&msg.content);
    
    if channels.is_empty() {
        msg.channel_id.send_message(ctx, |m| {
            m.content("Please *mention* some channel to be watched")
        }).await?;
        return Ok(());
    }
    
    let config = crate::read_config().await;
    let (description, field) = match config.guilds.get_mut(&guild_id) {
        Some(ref mut g) => {
            let removed_channels = channels
                .into_iter()
                .filter_map(|v| g.remove_sadkaede_channel(v))
                .collect::<Vec<_>>();
                
            if removed_channels.is_empty() {
                ("These channels don't exist in the list...".to_string(), None)
            } else {
                update_guild_config(&ctx, &g).await?;
                let mess = format!("Removed {} channels", removed_channels.len());
                let s = removed_channels
                    .into_iter()
                    .map(|v| format!("<#{}>", v.0))
                    .join(" ");
                    
                (mess, Some(s))
            }
        }
        
        None => ("These channels don't exist in the list...".to_string(), None),
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
            embed.field("Channels", value, true);
        }
        
        embed
    })).await?;
    
    Ok(())
}