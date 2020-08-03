use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    avatar,
    ping,
    say,
    choose,
    uptime,
    kanji,
    urban,
    love,
    info
}

#[group]
#[commands(avatar, say, love, choose, ping, kanji, urban, info, uptime)]
struct General;
