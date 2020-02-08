use crate::commands::prelude::*;
use crate::utils::send_file;

#[command]
fn diancie(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    send_file(&ctx.http, msg.channel_id, "./static/img/Diancie.gif");

    Ok(())
}
