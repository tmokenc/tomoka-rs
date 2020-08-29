use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    add,
    remove,
    count,
    evidence,
}

#[group]
#[prefix("rgb")]
#[default_command(evidence)]
#[commands(add, remove, count, evidence)]
#[owner_privilege]
struct RGB;

