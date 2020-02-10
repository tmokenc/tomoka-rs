use crate::commands::prelude::*;
use crate::types::GuildConfig;

#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
#[bucket = "basic"]
/// Change the logging channel
/// **NOTE**: This command will enable the logger no matter what
fn channel(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let channel = match extract_channel_ids(&msg.content).into_iter().next() {
        Some(c) => c,
        None => return Ok(())
    };
        
    let config = crate::read_config();
    let mut guild = config
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(guild_id.0));
        
    let old_channel = guild.set_log_channel(channel.0);
    guild.enable_logger();
    
    update_guild_config(&ctx, &guild)?;

    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Logger information");
        embed.color(config.color.information);
        embed.timestamp(now());
        
        match old_channel {
            Some(c) if c == channel => {
                embed.description(format!("Seems like the logger is already on <#{}>", c));
            }
            
            Some(c) => {
                embed.description("Changed the logging channel");
                embed.field("New channel", format!("<#{}>", channel), true);
                embed.field("Old channel", format!("<#{}>", c), true);
            }
            
            _ => {
                embed.description(format!("Enabled the logger to be logged on <#{}>", channel));
            } 
        }
        
        embed
    }))?;

    Ok(())
}