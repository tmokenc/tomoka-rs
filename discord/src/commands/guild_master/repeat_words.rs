use magic::import_all;
use serenity::framework::standard::macros::group;

import_all! {
    add,
    remove,
    enable,
    disable,
    toggle,
    info
}

#[group]
#[prefixes("repeat_word", "repeat_words", "words", "word")]
#[commands(add, remove, enable, disable, toggle, info)]
#[default_command(info)]
struct RepeatWords;