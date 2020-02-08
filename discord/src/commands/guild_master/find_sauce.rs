use magic::import_all;
use serenity::framework::standard::macros::group;

import_all! {
    info,
    add,
    remove,
    enable,
    disable,
    all,
    disable_all,
    toggle
}

#[group]
#[prefixes("findsauce", "sauce")]
#[commands(info, add, remove, enable, disable, all, disable_all, toggle)]
#[default_command(info)]
struct FindSauce;
