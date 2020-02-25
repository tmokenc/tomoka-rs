use std::future::Future;
use std::path::Path;
use std::sync::Arc;

use bytes::Bytes;
use lazy_static::lazy_static;
use log::error;
use magic::traits::MagicIter;
use magic::{number_to_le_bytes, number_to_rgb};
use parking_lot::RwLock;
use regex::Regex;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Runtime;

use colorful::core::color_string::CString;
use colorful::Colorful;
use colorful::RGB;

use crate::storages::*;
use crate::types::GuildConfig;
use crate::Result;

use serenity::{
    builder::CreateEmbed,
    client::bridge::voice::ClientVoiceManager,
    client::Context,
    http::{AttachmentType, Http},
    model::{
        id::{ChannelId, GuildId, UserId},
        user::User,
    },
    prelude::TypeMapKey,
};

pub type Color = (u8, u8, u8);

lazy_static! {
    pub static ref TOKIO_RT: RwLock<Runtime> = RwLock::new(Runtime::new().unwrap());
}

/// Check if a (guild) channel is nsfw or not
pub fn is_nsfw_channel<C: Into<ChannelId>>(ctx: &Context, channel: C) -> bool {
    channel
        .into()
        .to_channel(ctx)
        .ok()
        .and_then(|v| v.guild())
        .filter(|v| v.read().nsfw)
        .is_some()
}

//I have problem with the built-in method `parse_channel` of the serenity, so I decided to write my own function for this.
pub fn extract_channel_ids(msg: &str) -> Vec<ChannelId> {
    lazy_static! {
        static ref CHANNEL_RE: Regex = Regex::new(r"<#(\d+)>").unwrap();
    }
    CHANNEL_RE
        .captures_iter(msg)
        .filter_map(|v| v[1].parse::<u64>().ok())
        .map(ChannelId)
        .collect()
}

/// removes mentions from the message
pub fn remove_mention<S: AsRef<str>>(msg: S) -> String {
    lazy_static! {
        static ref MENTION_RE: Regex = Regex::new("<@[0-9]+>").unwrap();
    }
    MENTION_RE.replace_all(msg.as_ref(), "").to_string()
}

/// removes all emote from the message
pub fn remove_emote<S: AsRef<str>>(msg: S) -> String {
    lazy_static! {
        static ref EMOTE_RE: Regex = Regex::new(r"<:\w+:[0-9]+>").unwrap();
    }
    EMOTE_RE.replace_all(msg.as_ref(), "").to_string()
}

pub fn parse_eh_token(content: &str) -> Vec<(u32, String)> {
    lazy_static! {
        static ref KAEDE_REG: Regex =
            Regex::new(r"e(x|\-)hentai.org/g/(\d+)/([[:alnum:]]+)").unwrap();
    }

    KAEDE_REG
        .captures_iter(content)
        .filter_map(|res| {
            res.get(2)
                .and_then(|v| v.as_str().parse::<u32>().ok())
                .and_then(|v| res.get(3).map(|x| (v, x)))
                .map(|(v, x)| (v, x.as_str().to_string()))
        })
        .collect()
}

pub fn colored_name_user(user: &User) -> CString {
    let (r, g, b) = number_to_rgb(user.id.0);
    let color = RGB::new(r, g, b);
    let name = format!("{}#{:04}", user.name.to_owned(), user.discriminator);

    name.color(color)
}

pub fn get_user_voice_channel(ctx: &Context, guild_id: GuildId, mem: UserId) -> Option<ChannelId> {
    guild_id
        .to_guild_cached(&ctx)
        .and_then(|v| {
            let guild = v.read();
            guild.voice_states.get(&mem).cloned()
        })
        .and_then(|v| v.channel_id)
}

pub fn is_playing(ctx: &Context, guild_id: GuildId) -> Option<ChannelId> {
    ctx.data
        .read()
        .get::<InforKey>()
        .and_then(|v| get_user_voice_channel(ctx, guild_id, v.user_id))
}

pub fn send<C: Into<ChannelId>, S: ToString, F>(
    http: impl AsRef<Http>,
    channel: C,
    content: S,
    embed_f: F,
) where
    F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
{
    let message = channel
        .into()
        .send_message(http, |m| m.content(content.to_string()).embed(embed_f));

    if let Err(why) = message {
        error!("Error while sending message...\n{:#?}", why);
    };
}

pub fn send_embed<C, F>(http: impl AsRef<Http>, channel: C, embed_f: F)
where
    C: Into<ChannelId>,
    F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
{
    let message = channel.into().send_message(http, |m| m.embed(embed_f));

    if let Err(why) = message {
        error!("Error while sending message...\n{:#?}", why);
    };
}

/// where F: Into<AttachmentType<'a>>
/// gonna remove this soon because the newer serenity now supported this
pub fn send_file<C: Into<ChannelId>, F: AsRef<str>>(http: impl AsRef<Http>, channel: C, file: F) {
    let mut to_send: Vec<AttachmentType> = Vec::new();
    let url = file.as_ref();

    let bytes: Bytes;
    if url.starts_with("http") {
        if let Ok(b) = get_file_bytes(url) {
            bytes = b;
            let name = url.split('/').last().unwrap();
            info!("file: {} | size: {}", name.to_owned(), bytes.len());
            to_send.push((bytes.as_ref(), name).into());
        }
    } else {
        to_send.push(file.as_ref().into());
    }

    if let Err(why) = channel.into().send_files(http, to_send, |m| m) {
        error!("Error while sending the file...\n{:#?}", why);
    }
}

#[inline]
pub fn get_guild_id_from_channel<C: Into<ChannelId>>(
    ctx: &Context,
    channel_id: C,
) -> Option<GuildId> {
    channel_id
        .into()
        .to_channel(ctx)
        .ok()
        .and_then(|v| v.guild())
        .map(|v| v.read().guild_id)
}

#[inline]
pub fn get_voice_manager(ctx: &Context) -> Arc<serenity::prelude::Mutex<ClientVoiceManager>> {
    get_data::<VoiceManager>(ctx).expect("Expected the voice manager")
}

/// This will clone the value if exist
#[inline]
pub fn get_data<D: TypeMapKey>(ctx: &Context) -> Option<D::Value>
where
    D::Value: Sync + Send + Clone,
{
    ctx.data.read().get::<D>().cloned()
}

// #[rustfmt_skip]
pub fn update_guild_config(ctx: &Context, new_config: &GuildConfig) -> Result<()> {
    let key = number_to_le_bytes(new_config.id);
    let config_db = get_data::<DatabaseKey>(ctx).unwrap().open("GuildConfig")?;
    if new_config.is_default() {
        config_db.delete(key)?;
    } else {
        config_db.put_json(key, new_config)?;
    }
    Ok(())
}

#[inline]
pub fn block_on<F: Future>(future: F) -> F::Output {
    TOKIO_RT.write().block_on(future)
}

#[inline]
pub fn spawn<F>(future: F)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    TOKIO_RT.read().spawn(future);
}

pub fn get_file_bytes(url: impl AsRef<str>) -> Result<Bytes> {
    let bytes = block_on(async { requester::get(url.as_ref()).await?.bytes().await })?;

    Ok(bytes)
}

pub fn save_file<P: AsRef<Path> + Send + 'static>(url: String, name: P) {
    use std::io::{Error, ErrorKind};

    spawn(async move {
        let mut file = File::create(name).await?;
        let mut stream = requester::get(&url)
            .await
            .map_err(|_| Error::new(ErrorKind::Other, "Reqwest Error..."))?;

        while let Some(data) = stream
            .chunk()
            .await
            .map_err(|_| Error::new(ErrorKind::BrokenPipe, "Reqwest error..."))?
        {
            file.write_all(&data).await?;
        }

        std::io::Result::<()>::Ok(())
    });
}

/// Get the dominant color from a url
pub fn get_dominant_color(url: &str) -> Result<Color> {
    let bytes = get_file_bytes(url)?;
    let colors = magic::image::get_dominanted_colors(bytes)?;
    let dominanted = colors.get(0).unwrap_or(&(255, 255, 255));
    Ok(*dominanted)
}

#[inline]
pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}
