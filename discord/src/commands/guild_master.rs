use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    repeat_words,
    find_sauce,
    prefix,
    logger
}

#[group]
#[prefixes("guild_option", "option", "opt")]
#[only_in("guilds")]
#[sub_groups(Prefix, Logger, FindSauce, RepeatWords)]
struct GuildMaster;
