use serenity::framework::standard::macros::group;
use magic::import_all;

const DB_KEY: &str = "Reminders";

import_all! {
    set,
    list,
    remove
}

#[group]
#[prefixes("remind_me", "remindme", "remind")]
#[commands(set, list, remove)]
#[default_command(set)]
struct Reminder;




