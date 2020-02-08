use crate::commands::prelude::*;
use crate::types::GuildConfig;

#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
#[bucket = "basic"]
/// Enable the logger
fn enable(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let channel = extract_channel_ids(&msg.content)
        .into_iter()
        .next();
        
    let mut guild = crate::read_config()
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(guild_id.0));

    let mess = match (guild.logger.enable, guild.logger.channel, channel) {
        (true, None, Some(new_channel)) => {
            guild.set_log_channel(new_channel.0);
            update_guild_config(&ctx, &guild)?;
            format!("The logger is enabled at <#{}>", new_channel)
        }
        
        (true, Some(channel), _) => {
            format!("The logger is already enabled at <#{}>", channel)
        }
        
        (false, _, Some(channel)) => {
            guild.set_log_channel(channel.0);
            guild.enable_logger();
            update_guild_config(&ctx, &guild)?;
            
            format!("The logger is enabled at <#{}>", channel.0)
        }
        
        _ => {
            String::from("The log channel has not been set.
            Mention a channel to set it up and enable the logger.")
        }
        
    };

    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Logger information");
        embed.color(INFORMATION_COLOR);
        embed.timestamp(now());
        
        embed.description(mess);
        
        embed
    }))?;

    Ok(())
}