use crate::commands::prelude::*;

#[command]
#[aliases("remove", "unset")]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
/// Clear the custom prefix if exists
fn clear(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config();
    
    let mut guild = config
        .guilds
        .get_mut(&guild_id);
        
    let description = match guild {
            Some(ref mut g) if g.prefix.is_some() => {
                g.remove_prefix();
                update_guild_config(&ctx, &g)?;
                "Removed the custom prefix"
            }
            
            _ => "Not found any custom prefix tho..."
        };
    
    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Prefix information");
        embed.color(config.color.information);
        embed.timestamp(now());
        
        embed.field("Current default prefix", &config.prefix, true);
        embed.description(description);
        embed
    }))?;
    
    Ok(())
}