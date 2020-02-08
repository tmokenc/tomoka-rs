use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    tmq
}

#[group]
#[commands(touhou_music_quiz)]
struct Game;
