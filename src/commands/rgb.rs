use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    add,
    add_member,
    remove,
    remove_member,
    count,
    evidence,
}

#[group]
#[prefix("rgb")]
#[default_command(evidence)]
#[commands(add, remove, add_member, remove_member, count, evidence)]
#[owner_privilege]
struct RGB;

