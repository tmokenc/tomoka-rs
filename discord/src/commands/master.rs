use magic::import_all;
use serenity::framework::standard::macros::group;

import_all! {
    say_in,
    set_cache_size,
    clear_cache,
    system_info,
    save_config,
    reload,
    restart,
    shutdown
}

#[group]
#[owners_only]
#[commands(
    say_in,
    set_cache_size,
    clear_cache,
    system_info,
    save_config,
    reload,
    restart,
    shutdown
)]
struct Master;
