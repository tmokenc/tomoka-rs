use crate::commands::prelude::*;
use crate::utils::{get_dominant_color, Color};
use futures::future::{self, TryFutureExt};

#[command]
#[aliases("ava")]
#[usage = "?[@someone]"]
#[example = "@SuperUser"]
/// Get the avatar of an user
/// If mention multiple users, I will take the first one
/// If none, I will response with the user's avatar
async fn avatar(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    msg.channel_id.broadcast_typing(&ctx).await?;
    let user = msg.mentions.get(0).unwrap_or(&msg.author);
    let display_name = format!("{}#{:04}", user.name, user.discriminator);
    let avatar = user
        .avatar_url()
        .unwrap_or_else(|| user.default_avatar_url());

    let static_avatar = {
        let mut a = avatar.split('.').collect::<Vec<_>>();
        a.pop();
        a.push("png");

        a.join(".")
    };
    
    let mess = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|embed| {
            embed
                .title(display_name.to_owned())
                .image(avatar.to_owned())
                .timestamp(Utc::now().to_rfc3339())
        })
    });
    
    let color = get_dominant_color(&static_avatar);
   
    let join = future::try_join(
        color,
        mess.map_err(|e| Box::new(e) as Box<_>)
    );
     
    if let Ok((color, mut message)) = join.await {
        info!("the dominanted color is {:?}", &color);
         
        message.edit(ctx, move |m| {
            m.embed(move |embed| {
                embed
                    .color(color)
                    .title(display_name)
                    .image(avatar)
                    .timestamp(Utc::now().to_rfc3339())
            })
        }).await?;
    }
    
    Ok(())
}
