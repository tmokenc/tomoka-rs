use crate::commands::prelude::*;

#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
/// Disable the repeat-words machine.
fn disable(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config();
    let mut guild = config
        .guilds
        .get_mut(&guild_id);
        
    if let Some(ref mut g) = guild {
        g.disable_repeat_words();
        update_guild_config(&ctx, &g)?;
    }

    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Repeat-words information");
        embed.color(config.color.information);
        embed.timestamp(now());
        
        embed.description("Disabled the repeat-words machine");
        embed
    }))?;
    
    Ok(())   
}