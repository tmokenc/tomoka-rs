use crate::commands::prelude::*;
use crate::traits::ChannelExt;

#[command]
#[aliases("information")]
async fn info(ctx: &Context, msg: &Message, _arg: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let user_info = data.get::<InforKey>().unwrap();
    let my_info = ctx.http.get_current_application_info().await?;

    let description = format!(
        "Hi, I'm {}\nCreated by: {}#{:04}\n{}",
        my_info.name, 
        my_info.owner.name, 
        my_info.owner.discriminator,
        my_info.description,
    );
    
    msg.channel_id.send_embed(ctx)
        .with_description(description)
        .with_field("Executed commands", user_info.executed_commands(), true)
        .await?;

    Ok(())
}
