use crate::commands::prelude::*;
use crate::types::GuildConfig;

#[command]
#[min_args(1)]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
/// Add (some) word to the repeat list
/// Seperate by `, `
/// These word will be repeated by the bot when someone use it
/// This command will automatically enable the repeat-word machine
async fn add(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let words = args.rest().split(", ").collect::<Vec<_>>();
    if words.get(0).filter(|x| !x.is_empty()).is_none() {
        return Ok(())
    }
    
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    
    let mut guild = config
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(guild_id.0));

    let length = guild.add_words(words);
    
    let color = config.color.information;
    let description = if length > 0 {
        guild.enable_repeat_words();
        update_guild_config(&ctx, &guild).await?;
        
        format!("Added {} words to be repeated", length)
    } else {
        String::from("These words are in the list already")
    };
    
    drop(guild);
    drop(config);

    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Repeat-words information");
        embed.color(color);
        embed.timestamp(now());
        
        embed.description(description);
        embed
    })).await?;
    
    
    Ok(())
}