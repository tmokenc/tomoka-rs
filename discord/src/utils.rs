use std::path::Path;

use bytes::Bytes;
use lazy_static::lazy_static;
use magic::{number_to_le_bytes, number_to_rgb};
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
    let channel = channel
        .into()
        .to_channel(ctx)
        .await;

    match channel {
        Ok(v) => v.is_nsfw().await,
        Err(_) => false,
    }
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

pub async fn get_user_voice_channel(ctx: &Context, guild_id: GuildId, mem: UserId) -> Option<ChannelId> {
    match guild_id.to_guild_cached(&ctx).await {
        Some(c) => {
            let guild = c.read().await;
            guild.voice_states.get(&mem).and_then(|v| v.channel_id)
        }
        None => None
    }
}

pub async fn is_playing(ctx: &Context, guild_id: GuildId) -> Option<ChannelId> {
    match ctx.data.read().await.get::<InforKey>() {
        Some(v) => get_user_voice_channel(ctx, guild_id, v.user_id).await,
        None => None,
    }
}

pub async fn is_dead_channel(ctx: &Context, channel_id: ChannelId) -> bool {
    let members = match channel_id.to_channel(ctx).await.ok().and_then(|v| v.guild()) {
        Some(g) => match g.read().await.members(ctx).await {
            Ok(m) => m,
            Err(_) => return false,
        }
        None => return false,
    };
    
    for member in members {
        if !member.user.read().await.bot {
            return false
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
        .ok()
        .and_then(|v| v.guild());

    match guild {
        Some(v) => Some(v.read().await.guild_id),
        None => None,
    }
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
    let key = number_to_le_bytes(new_config.id);
    let config_db = get_data::<DatabaseKey>(ctx)
        .await
        .unwrap()
        .open("GuildConfig")?;

    tokio::task::block_in_place(move || {
        if new_config.is_default() {
            config_db.delete(key)
        } else {
            config_db.put_json(key, new_config)
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
    let colors = tokio::task::block_in_place(move || magic::image::get_dominanted_colors(bytes))?;
    let dominanted = colors.get(0).unwrap_or(&(255, 255, 255));
    Ok(*dominanted)
}

#[inline]
pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}
