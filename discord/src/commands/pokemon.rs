use magic::import_all;
use serenity::framework::standard::macros::group;

import_all! {
    strategy,
    nature,
}

#[group]
#[commands(nature, smogon_strategy)]
struct Pokemon;
