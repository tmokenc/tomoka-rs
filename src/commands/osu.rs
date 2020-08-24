use magic::import_all;
use serenity::framework::standard::macros::group;

import_all! {
    osu_match,
}

#[group]
#[commands(osu_match)]
struct Osu;
