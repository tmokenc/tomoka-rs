use crate::commands::prelude::*;
use requester::GoogleScraper as _;

#[command]
#[aliases("search", "g")]
async fn google(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let text = args.rest();
    let data = get_data::<ReqwestClient>(&ctx)
        .await
        .unwrap()
        .search(&text)
        .await?;
        
    let color = crate::read_config().await.color.information;
    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.color(color);
        embed.title(format!("Result for `{}`", text));
        
        for value in data {
            let title = format!("[{}]({})", value.title, value.link);
            embed.field(title, value.description, false);
        }
        
        embed
    })).await?;
    
    Ok(())
}