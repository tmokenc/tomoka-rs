use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    prune
}

#[group]
#[commands(prune)]
struct Administration;
