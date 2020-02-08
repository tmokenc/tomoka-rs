use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    set_cache_size,
    clear_cache,
    system_info,
    save_config,
    restart,
    shutdown
}

#[group]
#[owners_only]
#[commands(set_cache_size, clear_cache, system_info, save_config, restart, shutdown)]
struct Master;