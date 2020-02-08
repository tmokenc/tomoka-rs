use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    strategy
}

#[group]
#[commands(smogon_strategy)]
struct Pokemon;
