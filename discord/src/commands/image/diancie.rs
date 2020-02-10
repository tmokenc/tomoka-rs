use crate::commands::prelude::*;
use crate::utils::send_file;

#[command]
/// Show an extremely cute Diancie (pokemon)
fn diancie(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    send_file(&ctx.http, msg.channel_id, "./assets/img/Diancie.gif");

    Ok(())
}
