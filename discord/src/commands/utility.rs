use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    ehentai,
    time
}

#[group]
#[commands(time)]
#[sub_groups(Ehentai)]
struct Utility;