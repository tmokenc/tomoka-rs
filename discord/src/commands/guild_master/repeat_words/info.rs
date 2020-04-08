use crate::commands::prelude::*;
use crate::traits::Embedable as _;

#[command]
#[only_in(guilds)]
/// Get the info of the repeat-words machine
async fn info(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let data = config
        .guilds
        .get(&guild_id)
        .map(|ref g| g.repeat_words.embed_data());
        
    let color = config.color.information;
    
    drop(config);
        
    msg.channel_id.send_message(ctx, |m| m.embed(|mut embed| {
        if let Some(value) = data {
            embed.0 = value;
        } else {
            embed.description("The repeat-words machine doesn't running on this guild yet");
        }
        
        embed.title("Repeat-words information");
        embed.color(color);
        embed.timestamp(now());
       
        embed
    })).await?;
    
    Ok(())
}