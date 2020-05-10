use magic::import_all;
use serenity::framework::standard::macros::group;

import_all! {
    ehentai,
    search,
    search_image,
    remind_me,
    time
}

#[group]
#[commands(search, search_image, remind_me, time)]
#[sub_groups(Ehentai)]
struct Utility;
