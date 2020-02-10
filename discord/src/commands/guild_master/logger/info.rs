use crate::commands::prelude::*;

#[command]
#[only_in(guilds)]
/// Information of the logger
fn info(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config();
    let guild = config
        .guilds
        .get(&guild_id);

    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Logger information");
        embed.color(config.color.information);
        embed.timestamp(now());
        
        match guild {
            Some(ref g) if g.logger.enable && g.logger.channel.is_some() => {
                let c = g.logger.channel.unwrap();
                embed.description(format!("The logger is on in <#{}>", c))
            }
            
            _ => embed.description("The logger is disabled")
        };
        
        embed
    }))?;

    Ok(())
}