use crate::commands::prelude::*;
use crate::utils::{get_dominant_color, Color};
use std::sync::mpsc::channel;
use std::thread;

#[command]
#[aliases("ava")]
#[usage = "?[@someone]"]
#[example = "@SuperUser"]
#[description = "Get the avatar of an user\n\
If mention multiple users, I will take the first one
If none, I will response with the user's avatar"]
fn avatar(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    msg.channel_id.broadcast_typing(&ctx)?;
    let user = msg.mentions.get(0).unwrap_or(&msg.author);
    let display_name = format!("{}#{:04}", user.name, user.discriminator);
    let avatar = user
        .avatar_url()
        .unwrap_or_else(|| user.default_avatar_url());

    let (sender, receiver) = channel::<Result<Color, &'static str>>();
    let static_avatar = {
        let mut a = avatar.split('.').collect::<Vec<_>>();
        a.pop();
        a.push("png");

        a.join(".")
    };

    thread::spawn(move || {
        let result = get_dominant_color(&static_avatar).map_err(|_| "Failed to process the image");

        sender.send(result).unwrap();
    });

    let mut message = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|embed| {
            embed
                .title(display_name.to_owned())
                .image(avatar.to_owned())
                .timestamp(Utc::now().to_rfc3339())
        })
    })?;

    let color = receiver.recv()??;
    info!("the dominanted color is {:?}", &color);

    message.edit(&ctx, |m| {
        m.embed(|embed| {
            embed
                .title(display_name)
                .color(color)
                .image(avatar)
                .timestamp(Utc::now().to_rfc3339())
        })
    })?;

    Ok(())
}
