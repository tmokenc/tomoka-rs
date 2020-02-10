use crate::commands::prelude::*;

#[command]
#[min_args(1)]
#[only_in(guilds)]
#[required_permissions(MANAGE_GUILD)]
/// "Remove words (seperate by `, `) in the repeating words list
fn remove(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let words: Vec<_> = args.rest().trim().split(", ").collect();
    if words.get(0).filter(|x| !x.is_empty()).is_none() {
        return Ok(())
    }
    
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config();
    let mut guild = config.guilds.get_mut(&guild_id);
    let description = match guild {
        Some(ref mut guild) => {
            let length = guild.remove_words(words);
            
            if length == 0 {
                String::from("There is no word to be removed")
            } else {
                update_guild_config(&ctx, &guild)?;
                format!("Removed {} words", length)
            }
        }
        
        _ => {
            String::from("This guilds hasn't used this feature yet.")
        }
    };
            
    msg.channel_id.send_message(&ctx, |m| m.embed(|embed| {
        embed.title("Repeat-words information");
        embed.color(config.color.information);
        embed.timestamp(now());
        
        embed.description(description);
        embed
    }))?;
   
    Ok(())
}