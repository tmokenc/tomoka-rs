use crate::commands::prelude::*;
use requester::GoogleScraper as _;

#[command]
#[aliases("g")]
/// Search gooogle
async fn search(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let text = args.rest();
    let data = get_data::<ReqwestClient>(&ctx)
        .await
        .unwrap()
        .google_search(&text)
        .await?;
        
    let color = crate::read_config().await.color.information;
    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.color(color);
        embed.title(format!("Result for `{}`", text));
        
        for value in data {
            let description = format!("{}\n[[Link]]({})", value.description, value.link);
            embed.field(value.title, description, false);
        }
        
        embed
    })).await?;
    
    Ok(())
}
