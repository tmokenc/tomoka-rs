use serenity::framework::standard::macros::group;
use magic::import_all;

import_all! {
    repeat_words,
    find_sauce,
    find_sadkaede,
    prefix,
    logger
}

#[group]
#[prefixes("guild_option", "option", "opt")]
#[only_in("guilds")]
#[owner_privilege]
#[sub_groups(Prefix, Logger, FindSauce, FindSadKaede, RepeatWords)]
struct GuildMaster;
