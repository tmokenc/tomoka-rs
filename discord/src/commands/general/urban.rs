use crate::commands::prelude::*;
use crate::storages::ReqwestClient;
use crate::UrbanRequester;

#[command]
#[aliases("u")]
#[usage = "?<words>"]
#[example = "waifu"]
#[description = "Search the UrbanDictionary for a meaning of a slang word"]
fn urban(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.broadcast_typing(&ctx)?;
    let word = args.rest();
    let reqwest = get_data::<ReqwestClient>(&ctx).unwrap();
    let result = block_on(if word.is_empty() {
        reqwest.get_random()
    } else {
        reqwest.search_word(&word)
    })?;

    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        let config = crate::read_config();
        
        match result.get(0) {
            Some(u) => {
                embed.title(format!("Definition of {}", &u.word));
                embed.description(&u.definition);
                embed.url(&u.permalink);
                embed.color(config.color.information);
                embed.author(|author| author.name(&u.author));
                embed.timestamp(u.written_on.to_owned());
                embed.field("Example", &u.example, false);
                embed.field(":thumbsup:", u.thumbs_up, true);
                embed.field(":thumbsdown:", u.thumbs_down, true);
            }
            
            None => {
                embed.title(format!("Definition of {}", word));
                embed.description("404 Not Found");
                embed.color(config.color.error);
            }
        }
        
        embed
    }))?;

    Ok(())
}
