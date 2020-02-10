use core::time::Duration;
use serenity::model::id::EmojiId;

pub const MAX_CACHE_MESSAGE: usize = 500;
pub const MAX_FILE_SIZE: u32 = 7654321;

pub const HEART_URL: &str = "https://i.imgur.com/YpRPxeS.png";
pub const IMAGE_SEARCH_DEPTH: u64 = 20;

pub const SAUCE_WAIT_DURATION: Duration = Duration::from_secs(30);
pub const SAUCE_EMOJI: EmojiId = EmojiId(673657518777434122);

// TouhouMusicQuiz constants
pub const TMQ_DURATION: f32 = 62.0;