use super::get_last_image_buf;
use crate::commands::prelude::*;
use magic::image::{self, RotateAngle};

#[command]
#[description = "Rotate the last image from 20 most recent message on the channel"]
fn rotate(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.channel_id.broadcast_typing(&ctx)?;
    let image_buf = get_last_image_buf(&ctx, &msg, IMAGE_SEARCH_DEPTH);

    if image_buf.is_none() {
        msg.channel_id
            .say(ctx, "Coudn't find a image in range of 10 messages")?;
        return Ok(());
    }

    let arg = args
        .single::<String>()
        .map(|v| v.to_lowercase())
        .unwrap_or_default();

    let angle = match arg.as_ref() {
        "90" | "90%" | "90°" | "left" => RotateAngle::Left,
        "180" | "180%" | "180°" | "rotate180" => RotateAngle::Rotate180,
        _ => RotateAngle::Right,
    };

    let image = image::rotate(image_buf.unwrap(), angle)?;
    msg.channel_id
        .send_files(ctx, vec![(image.as_slice(), "res.png")], |m| m)?;
    Ok(())
}
