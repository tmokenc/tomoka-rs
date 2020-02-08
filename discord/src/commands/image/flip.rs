use super::get_last_image_buf;
use crate::commands::prelude::*;
use magic::image::{self, FlipType};

#[command]
#[description = "Flip the last image from last 20 messages on the channel"]
fn flip(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.channel_id.broadcast_typing(&ctx)?;
    let buf = get_last_image_buf(&ctx, &msg, IMAGE_SEARCH_DEPTH);

    if buf.is_none() {
        msg.channel_id
            .say(ctx, "Coudn't find an image in the most recent messages")?;
        return Ok(());
    }

    let arg = args
        .single::<String>()
        .map(|v| v.to_lowercase())
        .unwrap_or_default();

    let direction = match arg.as_ref() {
        "left" | "right" | "vertical" | "vertically" | "v" => FlipType::Vertical,
        _ => FlipType::Horizontal,
    };

    let data = image::flip(buf.unwrap(), direction)?;
    msg.channel_id
        .send_files(ctx, vec![(data.as_slice(), "res.png")], |m| m)?;
    Ok(())
}
