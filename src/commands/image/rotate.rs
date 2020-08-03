use super::get_last_image_buf;
use crate::commands::prelude::*;
use magic::image::{self, RotateAngle};

#[command]
/// Rotate the last image from 20 most recent message on the channel
async fn rotate(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let depth = crate::read_config().await.image_search_depth;
    let image_buf = match get_last_image_buf(&ctx, &msg, depth).await {
        Some(g) => g,
        None => {
            msg.channel_id
                .say(ctx, "Coudn't find a image in range of 10 messages").await?;
            return Ok(());
        }
    };

    let arg = args
        .single::<String>()
        .map(|v| v.to_lowercase())
        .unwrap_or_default();

    let angle = match arg.as_ref() {
        "90" | "90%" | "90°" | "left" => RotateAngle::Left,
        "180" | "180%" | "180°" | "rotate180" => RotateAngle::Rotate180,
        _ => RotateAngle::Right,
    };

    let image = tokio::task::spawn_blocking(|| image::rotate(image_buf, angle)).await??;
    msg.channel_id.send_files(ctx, vec![(image.as_slice(), "res.png")], |m| m).await?;
    Ok(())
}
