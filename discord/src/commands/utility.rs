use magic::import_all;
use serenity::framework::standard::macros::group;

import_all! {
    ehentai,
    search,
    search_image,
    reminder,
    time
}

#[group]
#[commands(search, search_image, time)]
#[sub_groups(Ehentai, reminder)]
struct Utility;
