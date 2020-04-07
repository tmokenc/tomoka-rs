use crate::commands::prelude::*;

#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
/// Disable the logger
async fn disable(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let mut guild = config
        .guilds
        .get_mut(&guild_id);

    let mess = match guild {
        Some(ref mut g) if g.logger.enable => {
            g.disable_logger();
            update_guild_config(&ctx, &g).await?;
            
            "Disabled the logger"
        }
        
        _ => "The logger is disabled already"
    };
    
    let color = config.color.information;
    
    drop(guild);
    drop(config);
    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Logger information");
        embed.color(color);
        embed.timestamp(now());
        
        embed.description(mess);
        
        embed
    })).await?;

    Ok(())
}