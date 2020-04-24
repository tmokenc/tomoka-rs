#![allow(unstable_name_collisions)]

use crate::traits::Embedable;
use crate::Result;
use chrono::{DateTime, Utc};
use core::time::Duration;
use serde::{Deserialize, Serialize};
use serenity::builder::CreateEmbed;
use serenity::http::client::Http;
use serenity::model::guild::Role;
use serenity::model::id::{ChannelId, GuildId, RoleId, UserId};
use smallstr::SmallString;
use std::collections::HashSet;
use std::default::Default;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

use magic::traits::MagicBool as _;
use magic::traits::MagicIter as _;

pub struct Information {
    pub booted_on: DateTime<Utc>,
    pub user_id: UserId,
    pub executed: AtomicUsize,
}

impl Information {
    pub async fn init(http: &Http) -> Result<Self> {
        let info = Self {
            booted_on: Utc::now(),
            user_id: http.get_current_user().await?.id,
            executed: AtomicUsize::new(0),
        };

        Ok(info)
    }

    #[inline]
    pub fn executed_commands(&self) -> usize {
        self.executed.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn executed_one(&self) -> usize {
        self.executed.fetch_add(1, Ordering::SeqCst)
    }

    pub fn uptime(&self) -> Duration {
        let current = Utc::now().timestamp_millis() as u64;
        let since = self.booted_on.timestamp_millis() as u64;

        Duration::from_millis(current - since)
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SimpleRole {
    pub name: SmallString<[u8; 32]>,
    pub id: RoleId,
    pub color: (u8, u8, u8),
}

impl From<Role> for SimpleRole {
    fn from(role: Role) -> Self {
        Self {
            id: role.id,
            name: SmallString::from(role.name),
            color: role.colour.tuple(),
        }
    }
}

impl fmt::Display for SimpleRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<@&{}>", self.id)
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DiscordLogger {
    pub enable: bool,
    pub channel: Option<ChannelId>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct FindSauce {
    pub all: bool,
    pub enable: bool,
    pub channels: HashSet<ChannelId>,
}

impl Embedable for FindSauce {
    fn append_to<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        if !self.enable || (self.channels.is_empty() && !self.all) {
            embed.description("The saucing service is disabled for this server");
        } else if self.all {
            embed.description("The saucing service is enabled for all channels on this server");
        } else {
            let mess = format!(
                "The saucing service is enabled for {} channels on this server",
                self.channels.len()
            );
            let s = self
                .channels
                .iter()
                .map(|v| format!("<#{}>", v.0))
                .join(" ");

            embed.description(mess);
            embed.field("Saucing channels", s, true);
        }

        embed
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindSadKaede {
    pub all: bool,
    pub enable: bool,
    pub channels: HashSet<ChannelId>,
}

impl Default for FindSadKaede {
    fn default() -> Self {
        Self {
            all: true,
            enable: true,
            channels: Default::default(),
        }
    }
}

impl Embedable for FindSadKaede {
    fn append_to<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        if !self.enable || (self.channels.is_empty() && !self.all) {
            embed.description("The SadKaede-finder service is disabled for this server");
        } else if self.all {
            embed.description(
                "The SadKaede-finder service is enabled for all channels on this server",
            );
        } else {
            let mess = format!(
                "The SadKaed-finder service is enabled for {} channels on this server",
                self.channels.len()
            );
            let s = self
                .channels
                .iter()
                .map(|v| format!("<#{}>", v.0))
                .join(" ");

            embed.description(mess);
            embed.field("SadKaede channels", s, true);
        }

        embed
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RepeatWords {
    pub enable: bool,
    pub words: HashSet<String>,
}

impl Embedable for RepeatWords {
    fn append_to<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        use magic::traits::MagicIter as _;

        if !self.enable {
            embed.description("Disabled the repeat-words machine");
        } else if self.words.is_empty() {
            embed.description(
                "Error 404: Word not found
            Use `option repeat_words add` command to add words to be repeated",
            );
        } else {
            let words = self.words.iter().map(|v| format!("`{}`", v)).join(", ");
            embed.description(format!(
                "This {} words are gonna be repeated when appear in the chat",
                self.words.len()
            ));
            embed.field("Words", words, false);
        }

        embed
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct GuildConfig {
    pub id: GuildId,
    pub prefix: Option<SmallString<[u8; 8]>>,
    pub rgblized: Option<Vec<SimpleRole>>,
    pub logger: DiscordLogger,
    pub find_sauce: FindSauce,
    pub find_sadkaede: FindSadKaede,
    pub repeat_words: RepeatWords,
}

impl GuildConfig {
    pub fn new<G: Into<GuildId>>(id: G) -> Self {
        Self {
            id: id.into(),
            ..Default::default()
        }
    }

    pub fn is_default(&self) -> bool {
        self.prefix.is_none()
            && self.logger.channel.is_none()
            && !self.logger.enable
            && self.find_sauce.channels.is_empty()
            && !self.find_sauce.all
            && !self.find_sauce.enable
            && self.find_sadkaede.all
            && self.find_sadkaede.enable
            && self.rgblized.is_none()
            && self.repeat_words.words.is_empty()
            && !self.repeat_words.enable
    }

    pub fn set_prefix<S: ToString>(&mut self, prefix: S) -> Option<SmallString<[u8; 8]>> {
        let old = self.prefix.clone();
        self.prefix = Some(SmallString::from(prefix.to_string()));
        old
    }

    pub fn remove_prefix(&mut self) -> Option<SmallString<[u8; 8]>> {
        let old = self.prefix.clone();
        self.prefix = None;
        old
    }

    pub fn set_log_channel<C: Into<ChannelId>>(&mut self, channel: C) -> Option<ChannelId> {
        let old = self.logger.channel;
        self.logger.channel = Some(channel.into());
        old
    }

    pub fn enable_logger(&mut self) {
        self.logger.enable = true;
    }

    pub fn disable_logger(&mut self) {
        self.logger.enable = false;
    }

    /// Toggle the logger on/off, return the new state
    pub fn toggle_logger(&mut self) -> bool {
        let status = !self.logger.enable;
        self.logger.enable = status;
        status
    }

    pub fn enable_find_sauce(&mut self) {
        self.find_sauce.enable = true;
    }

    pub fn disable_find_sauce(&mut self) {
        self.find_sauce.enable = false;
    }

    pub fn add_sauce_channel<C: Into<ChannelId>>(&mut self, channel: C) {
        let channel = channel.into();
        self.find_sauce.channels.insert(channel);
    }

    pub fn remove_sauce_channel<C: Into<ChannelId>>(&mut self, channel: C) -> Option<ChannelId> {
        let channel = channel.into();
        self.find_sauce.channels.remove(&channel).then_some(channel)
    }

    pub fn enable_find_sadkaede(&mut self) {
        self.find_sadkaede.enable = true;
    }

    pub fn disable_find_sadkaede(&mut self) {
        self.find_sadkaede.enable = false;
    }

    pub fn add_sadkaede_channel<C: Into<ChannelId>>(&mut self, channel: C) {
        let channel = channel.into();
        self.find_sadkaede.channels.insert(channel);
    }

    pub fn remove_sadkaede_channel<C: Into<ChannelId>>(&mut self, channel: C) -> Option<ChannelId> {
        let channel = channel.into();
        self.find_sadkaede
            .channels
            .remove(&channel)
            .then_some(channel)
    }

    /// Add roles to RGB, return the count of added roles
    pub fn add_rgb<I>(&mut self, roles: I) -> u8
    where
        I: IntoIterator<Item = Role>,
    {
        let rgb = self.rgblized.get_or_insert_with(Vec::new);
        let roles = roles
            .into_iter()
            .filter(|v| v.mentionable && rgb.iter().all(|x| v.id != x.id))
            .map(SimpleRole::from)
            .collect::<Vec<_>>();
        let length = roles.len() as u8;
        rgb.extend(roles);

        // sort by luminosity
        rgb.sort_by(|a, b| {
            fn get_luminosity((r, g, b): (u8, u8, u8)) -> f64 {
                let r = r as f64 * 0.2126;
                let g = g as f64 * 0.7152;
                let b = b as f64 * 0.0722;

                (r + g + b).sqrt()
            }

            let la = get_luminosity(a.color);
            let lb = get_luminosity(b.color);

            la.partial_cmp(&lb).unwrap_or(core::cmp::Ordering::Equal)
        });

        length
    }

    pub fn remove_rgb<I, N>(&mut self, roles: I) -> u8
    where
        I: IntoIterator<Item = N>,
        N: Into<RoleId>,
    {
        let mut length = 0u8;

        if let Some(v) = self.rgblized.as_mut() {
            let old_len = v.len();
            let roles: Vec<_> = roles.into_iter().map(|x| x.into()).collect();

            v.retain(|x| !roles.contains(&x.id));
            length = (old_len - v.len()) as u8;

            if v.is_empty() {
                self.rgblized = None;
            }
        }

        length
    }

    pub fn add_words<I, S>(&mut self, words: I) -> u8
    where
        I: IntoIterator<Item = S>,
        S: ToString,
    {
        let mut added = 0u8;

        for word in words {
            let w = word.to_string();

            if w.is_empty() {
                continue;
            }

            if self.repeat_words.words.insert(w) {
                added += 1;
            }
        }

        added
    }

    pub fn remove_words<I, S>(&mut self, words: I) -> u8
    where
        I: IntoIterator<Item = S>,
        S: ToString,
    {
        if self.repeat_words.words.is_empty() {
            return 0;
        }

        let mut removed = 0u8;

        for word in words {
            let w = word.to_string();
            if self.repeat_words.words.remove(&w) {
                removed += 1;
            }
        }

        removed
    }

    #[inline]
    pub fn enable_repeat_words(&mut self) {
        self.repeat_words.enable = true;
    }

    #[inline]
    pub fn disable_repeat_words(&mut self) {
        self.repeat_words.enable = false;
    }

    /// Toggle the repeat words, return the current state.
    pub fn toggle_repeat_words(&mut self) -> bool {
        self.repeat_words.enable = !self.repeat_words.enable;
        self.repeat_words.enable
    }
}

// pub enum PlayingSignal {
//     Resume,
//     Pause,
//     Stop,
//     Skip,
//     Previous,
//     Shuffle,
// }

// pub trait PlayOption: Send {
//     fn pause(&self);
//     fn resume(&self);
//     fn stop(&self);
//     fn skip(&self);
//     fn previous(&self);
//     fn shuffle(&self);
// }

// pub trait Playable: Send {
//     fn get_info(&self) -> Result<PlayingInfo>;
//     // fn play(&self, ctx: &Context, channel: ChannelId, signal: Receiver<PlayingSignal>);
// }

// #[derive(Default)]
// pub struct PlayingInfo {
//     pub title: String,
//     pub album_art: Option<String>,
//     pub artist: Option<String>,
//     pub album: Option<String>,
//     pub year: Option<String>,
//     pub requested_by: Option<String>,
// }

// pub struct Music {
//     sender: Sender<PlayingSignal>,
//     pub voice_channel: ChannelId,
//     pub music: Mutex<Box<dyn Playable>>,
// }

// impl Music {
//     pub fn new<M: Playable + 'static>(
//         voice_channel: ChannelId,
//         music: M,
//     ) -> (Self, Receiver<PlayingSignal>) {
//         let (sender, receiver) = mpsc::channel();
//         let s = Self {
//             sender,
//             voice_channel,
//             music: Mutex::new(Box::new(music)),
//         };

//         (s, receiver)
//     }
// }

// impl PlayOption for Music {
//     fn pause(&self) {
//         self.sender.send(PlayingSignal::Pause).ok();
//     }

//     fn resume(&self) {
//         self.sender.send(PlayingSignal::Resume).ok();
//     }

//     fn stop(&self) {
//         self.sender.send(PlayingSignal::Stop).ok();
//     }

//     fn skip(&self) {
//         self.sender.send(PlayingSignal::Skip).ok();
//     }

//     fn previous(&self) {
//         self.sender.send(PlayingSignal::Previous).ok();
//     }

//     fn shuffle(&self) {
//         self.sender.send(PlayingSignal::Shuffle).ok();
//     }
// }
