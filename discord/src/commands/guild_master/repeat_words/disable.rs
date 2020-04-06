use crate::commands::prelude::*;

#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
/// Disable the repeat-words machine.
async fn disable(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let mut guild = config
        .guilds
        .get_mut(&guild_id);
        
    if let Some(ref mut g) = guild {
        g.disable_repeat_words();
        update_guild_config(&ctx, &g).await?;
    }
    
    let color = config.color.information;
    
    drop(guild);
    drop(config);

    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Repeat-words information");
        embed.color(color);
        embed.timestamp(now());
        
        embed.description("Disabled the repeat-words machine");
        embed
    })).await?;
    
    Ok(())   
}