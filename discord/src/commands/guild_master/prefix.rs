use magic::import_all;
use serenity::framework::standard::macros::group;

import_all! {
    change,
    clear,
    info
}

#[group]
#[prefix = "prefix"]
#[commands(change, clear, info)]
#[default_command(info)]
struct Prefix;