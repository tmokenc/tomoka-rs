use crate::commands::prelude::*;

#[command]
/// Invite me to your server
async fn invite(ctx: &Context, msg: &Message) -> CommandResult {
    let url = format!(
        "https://discordapp.com/oauth2/authorize?client_id={}&scope=bot&permissions=8",
        ctx.data.read().await.get::<InforKey>().unwrap().user_id
    );
    
    msg.channel_id.say(ctx, url).await?;
    Ok(())
}