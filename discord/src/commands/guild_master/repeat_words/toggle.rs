use crate::commands::prelude::*;
use crate::types::GuildConfig;
use magic::traits::MagicIter;

#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
/// Toggle the repeat-words machine on/off
fn toggle(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(()),
    };
    
    let mut guild = crate::read_config()
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(guild_id.0));
    
    let current = guild.toggle_repeat_words();
    update_guild_config(&ctx, &guild)?;
    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Repeat-words information");
        embed.color(INFORMATION_COLOR);
        embed.timestamp(now());
        
        match (current, guild.repeat_words.words.len()) {
            (true, 0) => {
                embed.description("Enabled the repeat-words machine but there is no word in the list yet
                Consider using the `option words add` command to add words to be repeated");
            }
            
            (true, _) => {
                let words = guild
                .repeat_words
                .words
                .iter()
                .map(|w| format!("`{}`", w))
                .join(", ");
                
                embed.description("Enabled the repeat-words machine");
                embed.field("Words", words, false);
            } 
            
            (false, _) => {
                embed.description("Disabled the repeat-words machine");
            }
        }
        
        embed
    }))?;

    Ok(())
}