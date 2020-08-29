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
    info,
    invite,
}

#[group]
#[commands(avatar, say, love, choose, ping, kanji, urban, invite, info, uptime)]
struct General;
