use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    info
}

/// Get information of a E-H or sadpanda gallery
#[group]
#[prefixes("e-h", "eh", "e-hentai", "sadpanda", "sadkaede")]
#[commands(info)]
#[default_command(info)]
struct Ehentai;