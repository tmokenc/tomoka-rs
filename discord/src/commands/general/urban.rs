use crate::commands::prelude::*;
use crate::storages::ReqwestClient;
use crate::UrbanRequester;

#[command]
#[aliases("u")]
#[usage = "?<words>"]
#[example = "waifu"]
/// Search the UrbanDictionary for a meaning of a slang word
async fn urban(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.broadcast_typing(&ctx).await?;
    let word = args.rest();
    let reqwest = get_data::<ReqwestClient>(&ctx).await.unwrap();
    let result = if word.is_empty() {
        reqwest.get_random().await?
    } else {
        reqwest.search_word(&word).await?
    };
    
    let config = crate::read_config().await;
    let err_color = config.color.error;
    let suc_color = config.color.information;
    drop(config);

    msg.channel_id.send_message(ctx, move |m| m.embed(|embed| {
        match result.get(0) {
            Some(u) => {
                embed.title(format!("Definition of {}", &u.word));
                embed.description(&u.definition);
                embed.url(&u.permalink);
                embed.color(suc_color);
                embed.author(|author| author.name(&u.author));
                embed.timestamp(u.written_on.to_owned());
                embed.field("Example", &u.example, false);
                embed.field(":thumbsup:", u.thumbs_up, true);
                embed.field(":thumbsdown:", u.thumbs_down, true);
            }
            
            None => {
                embed.title(format!("Definition of {}", word));
                embed.description("404 Not Found");
                embed.color(err_color);
            }
        }
        
        embed
    })).await?;

    Ok(())
}
