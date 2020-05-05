use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    ehentai,
    search,
    search_image,
    time
}

#[group]
#[commands(time, search, search_image)]
#[sub_groups(Ehentai)]
struct Utility;