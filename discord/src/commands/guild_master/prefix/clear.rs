use crate::commands::prelude::*;

#[command]
#[aliases("remove", "unset")]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
/// Clear the custom prefix if exists
async fn clear(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    
    let mut guild = config
        .guilds
        .get_mut(&guild_id);
        
    let color = config.color.information;
    let prefix = config.prefix.to_owned();
    let description = match guild {
        Some(ref mut g) if g.prefix.is_some() => {
            g.remove_prefix();
            update_guild_config(&ctx, &g).await?;
            "Removed the custom prefix"
        }
        
        _ => "Not found any custom prefix tho..."
    };
    
    drop(guild);
    drop(config);
    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Prefix information");
        embed.color(color);
        embed.timestamp(now());
        
        embed.field("Current default prefix", prefix, true);
        embed.description(description);
        embed
    })).await?;
    
    Ok(())
}