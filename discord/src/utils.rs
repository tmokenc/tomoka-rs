use std::path::Path;
use std::sync::Arc;

use bytes::Bytes;
use lazy_static::lazy_static;
use magic::number_to_rgb;
use regex::Regex;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use colorful::core::color_string::CString;
use colorful::Colorful;
use colorful::RGB;

use crate::storages::*;
use crate::types::GuildConfig;
use crate::Result;

use serenity::{
    client::Context,
    model::{
        id::{ChannelId, GuildId, UserId},
        user::User,
    },
    prelude::TypeMapKey,
};

pub type Color = (u8, u8, u8);

#[derive(Debug)]
pub struct EmptyError;

impl std::fmt::Display for EmptyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An empty error, nothing to display here")
    }
}

impl std::error::Error for EmptyError {}
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

/// Check if a (guild) channel is nsfw or not
pub async fn is_nsfw_channel<C: Into<ChannelId>>(ctx: &Context, channel: C) -> bool {
    channel
        .into()
        .to_channel(ctx)
        .await
        .ok()
        .filter(|v| v.is_nsfw())
        .is_some()
}

#[inline]
pub fn typing(ctx: &Context, channel_id: ChannelId) {
    let http = Arc::clone(&ctx.http);
    tokio::spawn(channel_id.broadcast_typing(http));
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

pub async fn get_user_voice_channel(
    ctx: &Context,
    guild_id: GuildId,
    mem: UserId,
) -> Option<ChannelId> {
    guild_id
        .to_guild_cached(&ctx)
        .await?
        .read()
        .await
        .voice_states
        .get(&mem)?
        .channel_id
}

pub async fn is_playing(ctx: &Context, guild_id: GuildId) -> Option<ChannelId> {
    let user_id = ctx.data.read().await.get::<InforKey>()?.user_id;
    get_user_voice_channel(ctx, guild_id, user_id).await
}

pub async fn is_dead_channel(ctx: &Context, channel_id: ChannelId) -> bool {
    let members = match channel_id
        .to_channel(ctx)
        .await
        .ok()
        .and_then(|v| v.guild())
    {
        Some(g) => match g.members(ctx).await {
            Ok(m) => m,
            Err(_) => return false,
        },
        None => return false,
    };

    for member in members {
        if !member.user.bot {
            return false;
        }
    }

    true
}

pub async fn get_guild_id_from_channel<C: Into<ChannelId>>(
    ctx: &Context,
    channel_id: C,
) -> Option<GuildId> {
    let guild = channel_id
        .into()
        .to_channel(ctx)
        .await
        .ok()?
        .guild()?
        .guild_id;

    Some(guild)
}

/// This will clone the value if exist
#[inline]
pub async fn get_data<D: TypeMapKey>(ctx: &Context) -> Option<D::Value>
where
    D::Value: Sync + Send + Clone,
{
    ctx.data.read().await.get::<D>().cloned()
}

// #[rustfmt_skip]
pub async fn update_guild_config(ctx: &Context, new_config: &GuildConfig) -> Result<()> {
    let key = new_config.id;
    let config_db = get_data::<DatabaseKey>(ctx)
        .await
        .unwrap()
        .open("GuildConfig")?;

    tokio::task::block_in_place(move || {
        if new_config.is_default() {
            config_db.remove(&key)
        } else {
            config_db.insert(&key, new_config)
        }
    })?;

    Ok(())
}

pub async fn get_file_bytes(url: impl AsRef<str>) -> Result<Bytes> {
    let bytes = requester::get(url.as_ref()).await?.bytes().await?;
    Ok(bytes)
}

pub async fn save_file<P: AsRef<Path>>(url: String, name: P) -> Result<()> {
    let mut file = File::create(name).await?;
    let mut stream = requester::get(&url).await?;

    while let Some(data) = stream.chunk().await? {
        file.write_all(&data).await?;
    }

    Ok(())
}

/// Get the dominant color from a url
pub async fn get_dominant_color(url: &str) -> Result<Color> {
    let bytes = get_file_bytes(url).await?;
    let colors =
        tokio::task::spawn_blocking(|| magic::image::get_dominanted_colors(bytes)).await??;
    let dominanted = colors.get(0).unwrap_or(&(255, 255, 255));
    Ok(*dominanted)
}

#[inline]
pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}
