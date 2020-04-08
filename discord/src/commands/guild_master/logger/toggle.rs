#![allow(unstable_name_collisions)]

use crate::commands::prelude::*;
use crate::types::GuildConfig;
use magic::traits::MagicBool as _;

#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
/// Toggle the logger on/off
async fn toggle(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let mut guild = config
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(guild_id.0));

    let status = guild
        .toggle_logger()
        .then(|| {
            match guild.logger.channel {
                Some(c) => format!("<#{}>", c),
                None => "The log channel has not been set \nPlease use `log_channel` command to set the log channel".to_owned()
            }
        });
        
    let mess = if status.is_some() { "Enabled" } else { "Disabled" };
    let color = config.color.information;

    update_guild_config(&ctx, &guild).await?;
    
    drop(guild);
    drop(config);

    msg.channel_id.send_message(&ctx.http, |m| m.embed(|embed| {
        embed.title("Logger information");
        embed.color(color);
        embed.timestamp(now());
        
        embed.description(format!("**{}** logger", mess));
        
        if let Some(channel) = status {
            embed.field("Current log channel", channel, false);
        }
        
        embed
    })).await?;

    Ok(())
}
