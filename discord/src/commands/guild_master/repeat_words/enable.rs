use crate::commands::prelude::*;
use crate::types::GuildConfig;
use magic::traits::MagicIter;

#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
/// Enable the repeat-word function
async fn enable(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let mut guild = config
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(guild_id.0));
        
    guild.enable_repeat_words();
    update_guild_config(&ctx, &guild).await?;
    
    let color = config.color.information;
    let (description, words) = if guild.repeat_words.words.is_empty() {
        let mess = "Enabled the repeat-words machine but there is no word in the list yet
        Consider using the `option words add` command to add words to be repeated";
        (mess, None)
    } else {
        let words = guild
            .repeat_words
            .words
            .iter()
            .map(|w| format!("`{}`", w))
            .join(", ");
            
        ("Enabled the repeat-words machine", Some(words))
    };
    
    drop(guild);
    drop(config);
    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Repeat-words information");
        embed.color(color);
        embed.timestamp(now());
        embed.description(description);
        
        if let Some(w) = words {
            embed.field("Words", w, true);
        }
            
        embed
    })).await?;
        
    Ok(())
}