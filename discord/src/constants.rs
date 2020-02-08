#![allow(dead_code)]

use core::time::Duration;
use serenity::model::id::EmojiId;

pub const MAX_CACHE_MESSAGE: usize = 500;
pub const MAX_FILE_SIZE: u32 = 7654321;

pub const RGB_TU: &[&str] = &[
    "tu", "rau", "gf", "girls", "gái", "gai", "girl", "em", "browse", "cua", "xinh",
];

pub const HEART_URL: &str = "https://i.imgur.com/YpRPxeS.png";
pub const IMAGE_SEARCH_DEPTH: u64 = 20;

pub const MESSAGE_UPDATE_COLOR: u32 = 0x4edb5f;
pub const MESSAGE_DELETE_COLOR: u32 = 0xdb5f4e;
pub const ERROR_COLOR: u32 = 0xff0033;
//pub const ERROR_COLOR2: u32 = 0xffbaba;
pub const INFORMATION_COLOR: u32 = 0x9966ff;

pub const LOVELY_COLOR: u32 = 0xfc2368;
pub const TIME_FORMAT: &str = "%d/%m/%Y %r";

pub const SAUCE_WAIT_DURATION: Duration = Duration::from_secs(30);
pub const SAUCE_EMOJI: EmojiId = EmojiId(673657518777434122);
// TouhouMusicQuiz constants
pub const TMQ_WAIT_TIME: u64 = 69;
pub const TMQ_DURATION: f32 = 62.0;
pub const TMQ_NORMAL_DURATION: f32 = 15.0;
pub const TMQ_HARD_DURATION: f32 = 10.0;
pub const TMQ_LUNATIC_DURATION: f32 = 5.0;
pub const TMQ_EXTRA_DURATION: f32 = 2.0;
