use crate::utils::get_file_bytes;
use bytes::Bytes;
use magic::import_all;
use serenity::client::Context;
use serenity::framework::standard::macros::group;
use serenity::model::channel::Message;

import_all! {
    rotate,
    flip,
    saucenao,
    diancie
}

#[group]
#[commands(rotate, flip, saucenao, diancie)]
struct Image;

/// Get the last image buf from most recent message on the channel
/// Max messages length is 100
pub async fn get_last_image_buf(ctx: &Context, msg: &Message, limit: u16) -> Option<Bytes> {
    let url = get_last_image_url(ctx, msg, limit).await?;
    get_file_bytes(url).await.ok()
}

pub async fn get_last_image_url(ctx: &Context, msg: &Message, limit: u16) -> Option<String> {
    match get_image_url_from_message(&msg) {
        None => msg.channel_id
            .messages(ctx, |m| m.limit(limit as u64).before(msg.id))
            .await
            .ok()?
            .into_iter()
            .find_map(|v| get_image_url_from_message(&v)),
        v => v,
    }
}

#[inline]
fn get_image_url_from_message(msg: &Message) -> Option<String> {
    msg.attachments
        .iter()
        .find(|v| v.width.is_some())
        .map(|v| v.url.to_owned())
        .or_else(|| {
            msg.embeds
                .iter()
                .find_map(|v| v.image.as_ref())
                .map(|v| v.url.to_owned())
        })
}
