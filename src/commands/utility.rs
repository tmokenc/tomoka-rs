use magic::import_all;
use serenity::framework::standard::macros::group;

import_all! {
    ehentai,
    nhentai,
    search,
    search_image,
    reminder,
    time,
    translate,
}

#[group]
#[commands(search, search_image, time, nhentai, translate)]
#[sub_groups(Ehentai, reminder)]
struct Utility;
