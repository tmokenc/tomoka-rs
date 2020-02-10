use crate::commands::prelude::*;

#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
#[bucket = "basic"]
/// Disable the logger
fn disable(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config();
    let mut guild = config
        .guilds
        .get_mut(&guild_id);

    let mess = match guild {
        Some(ref mut g) if g.logger.enable => {
            g.disable_logger();
            update_guild_config(&ctx, &g)?;
            
            "Disabled the logger"
        }
        
        _ => "The logger is disabled already"
    };
    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Logger information");
        embed.color(config.color.information);
        embed.timestamp(now());
        
        embed.description(mess);
        
        embed
    }))?;

    Ok(())
}