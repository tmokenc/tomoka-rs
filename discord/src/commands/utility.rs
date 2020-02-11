use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    ehentai
}

#[group]
#[sub_groups(Ehentai)]
struct Utility;