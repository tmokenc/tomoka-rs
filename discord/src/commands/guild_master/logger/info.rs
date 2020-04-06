use crate::commands::prelude::*;

#[command]
#[only_in(guilds)]
/// Information of the logger
async fn info(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(()),
    };

    let config = crate::read_config().await;
    let log_channel = config
        .guilds
        .get(&guild_id)
        .as_deref()
        .filter(|v| v.logger.enable)
        .and_then(|v| v.logger.channel);
        
    let color = config.color.information;
    drop(config);

    msg.channel_id.send_message(ctx, |m| {
        m.embed(|embed| {
            embed.title("Logger information");
            embed.color(color);
            embed.timestamp(now());

            match log_channel {
                Some(channel) => embed.description(format!("The logger is on in <#{}>", channel)),
                None => embed.description("The logger is disabled"),
            };

            embed
        })
    }).await?;

    Ok(())
}
