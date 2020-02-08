use magic::import_all;
use serenity::framework::standard::macros::group;

import_all! {
    enable,
    disable,
    toggle,
    channel,
    info
}

#[group]
#[prefixes("log", "logger")]
#[commands(enable, disable, toggle, channel, info)]
#[default_command(info)]
struct Logger;