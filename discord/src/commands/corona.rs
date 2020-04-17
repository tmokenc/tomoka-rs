use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    leaderboard,
}

#[group]
#[commands(leaderboard)]
struct Corona;
