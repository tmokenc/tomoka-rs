use super::get_last_image_url;
use crate::commands::prelude::*;
use crate::traits::ToEmbed as _;
use magic::sauce::SauceNao;

#[command]
#[aliases("sauce")]
#[description = "Find an anime image source."]
fn saucenao(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.broadcast_typing(&ctx)?;
    let img = match get_last_image_url(&ctx, &msg, IMAGE_SEARCH_DEPTH) {
        Some(i) => i,
        None => {
            msg.channel_id.say(
                ctx,
                format!(
                    "Cannot find an image from last {} message",
                    IMAGE_SEARCH_DEPTH
                ),
            )?;
            return Ok(());
        }
    };

    let similarity = args.raw().find_map(|v| {
        if v.ends_with("%") {
            v[..v.len() - 1].parse::<f32>().ok()
        } else {
            None
        }
    });

    let data = SauceNao::get(&img, similarity)?;

    if data.not_found() {
        msg.channel_id.say(ctx, "Error 404: No sauce found")?;
        return Ok(());
    }
    
    msg.channel_id.send_message(ctx, |m| m.embed(|mut embed| {
        data.to_embed(&mut embed);
        embed
    }))?;

    Ok(())
}
