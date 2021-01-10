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
    corona,
    genshin,
}

#[group]
#[commands(search, search_image, time, nhentai, corona, translate)]
#[sub_groups(Ehentai, reminder, genshin)]
struct Utility;
