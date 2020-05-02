use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    ehentai,
    google,
    time
}

#[group]
#[commands(time, google)]
#[sub_groups(Ehentai)]
struct Utility;