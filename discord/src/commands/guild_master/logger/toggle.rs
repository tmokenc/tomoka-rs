use crate::commands::prelude::*;
use crate::types::GuildConfig;

#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
#[bucket = "basic"]
/// Toggle the logger on/off
fn toggle(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let mut guild_config = crate::read_config()
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(guild_id.0));

    let status = guild_config.toggle_logger();
    let mess = if status { "Enabled" } else { "Disabled" };

    update_guild_config(&ctx, &guild_config)?;

    msg.channel_id.send_message(&ctx.http, |m| m.embed(|embed| {
        embed.title("Logger information");
        embed.color(INFORMATION_COLOR);
        embed.timestamp(now());
        
        embed.description(format!("**{}** logger", mess));
        
        if status {
            let log_channel = match guild_config.logger.channel {
                Some(c) => format!("<#{}>", c),
                None => "The log channel has not been set \nPlease use `log_channel` command to set the log channel".to_owned()
            };
            embed.field("Current log channel", log_channel, false);
        }
        
        embed
    }))?;

    Ok(())
}
