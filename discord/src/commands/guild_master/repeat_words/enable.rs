use crate::commands::prelude::*;
use crate::types::GuildConfig;
use magic::traits::MagicIter;

#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
/// Enable the repeat-word function
fn enable(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config();
    let mut guild = config
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(guild_id.0));
        
    guild.enable_repeat_words();
    update_guild_config(&ctx, &guild)?;
    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Repeat-words information");
        embed.color(config.color.information);
        embed.timestamp(now());
        
        if guild.repeat_words.words.is_empty() {
            embed.description("Enabled the repeat-words machine but there is no word in the list yet
            Consider using the `option words add` command to add words to be repeated");
        } else {
            let words = guild
                .repeat_words
                .words
                .iter()
                .map(|w| format!("`{}`", w))
                .join(", ");
                
            embed.description("Enabled the repeat-words machine");
            embed.field("Words", words, false);
        }
            
        embed
    }))?;
        
    Ok(())
}