use crate::commands::prelude::*;

#[command]
#[only_in(guilds)]
/// Check prefix info on this server
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(()),
    };

    let config = crate::read_config().await;

    let prefix = config
        .guilds
        .get(&guild_id)
        .and_then(|g| g.prefix.to_owned())
        .unwrap_or_else(|| config.prefix.to_owned());

    let color = config.color.information;
    drop(config);

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|embed| {
                embed.title("Prefix information");
                embed.color(color);
                embed.timestamp(now());

                embed.description(format!("Current prefix is **__{}__**", prefix));
                embed
            })
        })
        .await?;

    Ok(())
}
