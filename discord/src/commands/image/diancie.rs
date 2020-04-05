use crate::commands::prelude::*;

#[command]
/// Show an extremely cute Diancie (pokemon)
async fn diancie(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    msg.channel_id.send_files(ctx, vec!["./assets/img/Diancie.gif"], |m| m).await?;
    Ok(())
}
