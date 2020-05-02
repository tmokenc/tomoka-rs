use super::get_last_image_buf;
use crate::commands::prelude::*;
use magic::image::{self, FlipType};

#[command]
/// Flip the last image from last 20 messages on the channel
async fn flip(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let depth = crate::read_config().await.image_search_depth;
    let buf = match get_last_image_buf(&ctx, &msg, depth).await {
        Some(b) => b,
        None => {
            msg.channel_id
                .say(ctx, "Coudn't find an image in the most recent messages").await?;
            return Ok(());
        }
    };

    let arg = args
        .single::<String>()
        .map(|v| v.to_lowercase())
        .unwrap_or_default();

    let direction = match arg.as_ref() {
        "left" | "right" | "vertical" | "vertically" | "v" => FlipType::Vertical,
        _ => FlipType::Horizontal,
    };

    let data = tokio::task::spawn_blocking(|| image::flip(buf, direction)).await??;
    msg.channel_id.send_files(ctx, vec![(data.as_slice(), "res.png")], |m| m).await?;
    Ok(())
}
