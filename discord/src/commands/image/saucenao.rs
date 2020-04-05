use super::get_last_image_url;
use crate::commands::prelude::*;
use crate::traits::ToEmbed as _;
use magic::sauce::SauceNao;

#[command]
#[aliases("sauce")]
/// Find an anime image source.
async fn saucenao(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let depth = crate::read_config().await.image_search_depth;
    let img = match get_last_image_url(&ctx, &msg, depth).await {
        Some(i) => i,
        None => {
            msg.channel_id.say(
                ctx,
                format!(
                    "Cannot find an image from last {} message",
                    depth
                ),
            ).await?;
            return Ok(());
        }
    };

    let similarity = args.raw().find_map(|v| {
        if v.ends_with('%') {
            v[..v.len() - 1].parse::<f32>().ok()
        } else {
            None
        }
    });

    let data = SauceNao::get(&img, similarity).await?;

    if data.not_found() {
        msg.channel_id.say(ctx, "Error 404: No sauce found").await?;
        return Ok(());
    }
    
    msg.channel_id.send_message(ctx, |m| m.embed(|mut embed| {
        data.to_embed(&mut embed);
        embed
    })).await?;

    Ok(())
}
