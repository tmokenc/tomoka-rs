use serenity::framework::standard::macros::group;
use serenity::model::event::Event;
use serenity::prelude::*;
use magic::import_all;
use dashmap::DashMap;

import_all! {
    pause,
    resume,
    stop,
    radio
}

// lazy_static::lazy_static! {
//     pub static ref PLAYING_LIST: DashMap<ChannelId, Music> = DashMap::new();
// }

#[group]
#[commands(radio, pause, resume, stop)]
struct Music;

// pub fn resume_plays_on_reconnect(ctx: &Context, ev: &Event) {
//     if let Event::Resumed(_) = ev {
//         // TODO
//     }
// }
