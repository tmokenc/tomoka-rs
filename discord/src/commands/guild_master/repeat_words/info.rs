use crate::commands::prelude::*;
use crate::traits::ToEmbed;

#[command]
#[only_in(guilds)]
/// Get the info of the repeat-words machine
fn info(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let guild = crate::read_config()
        .guilds
        .get(&guild_id);
        
    msg.channel_id.send_message(ctx, |m| m.embed(|mut embed| {
        embed.title("Repeat-words information");
        embed.color(INFORMATION_COLOR);
        embed.timestamp(now());
        
        match guild {
            Some(ref g) => {
                g.repeat_words.to_embed(&mut embed);
            },
            None => {
                embed.description("The repeat-words machine doesn't running on this guild yet");
            }
        }
        
        embed
    }))?;
    
    Ok(())
}