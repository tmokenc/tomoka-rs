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

    match result.get(0) {
        Some(u) => {
            msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|embed| {
                    embed
                        .title(format!("Definition of {}", &u.word))
                        .description(&u.definition)
                        .url(&u.permalink)
                        .color(INFORMATION_COLOR)
                        .author(|author| author.name(&u.author))
                        .timestamp(u.written_on.to_owned())
                        .field("Example", &u.example, false)
                        .field(":thumbsup:", u.thumbs_up, true)
                        .field(":thumbsdown:", u.thumbs_down, true)
                })
            })?;
        }
        None => {
            msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|embed| {
                    embed
                        .title(format!("Definition of {}", word))
                        .description("404 Not Found")
                        .color(ERROR_COLOR)
                        .timestamp(Utc::now().to_rfc3339())
                })
            })?;
        }
    }

    Ok(())
}
