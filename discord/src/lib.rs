#![allow(clippy::unreadable_literal)]

#[macro_use]
extern crate log;

extern crate config as lib_config;

mod cache;
mod commands;
mod config;
mod events;
mod framework;
mod global;
mod logger;
mod storages;
mod traits;
mod types;
mod utils;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub use requester::*;

use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use crate::config::Config;
use cache::MyCache;
use db::DbInstance;
use events::{Handler, RawHandler};
use framework::get_framework;
use magic::dark_magic::{bytes_to_le_u64, has_external_command};
use parking_lot::Mutex;
use serenity::Client;
use storages::*;
use types::*;

use colorful::{Color, Colorful};
use dotenv::dotenv;
use eliza::Eliza;

pub fn start() -> Result<()> {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN")?;

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    // env_logger::init();
    logger::init();

    let handler = Handler::new();
    let raw_handler = RawHandler::new();
    let custom_events_arc = raw_handler.custom_events.clone();

    info!(
        "Login with the token:\n{}",
        token.to_owned().underlined().yellow()
    );
    let mut client = Client::new_with_extras(&token, |e| {
        e.event_handler(handler).raw_event_handler(raw_handler)
    })?;

    // Disable the default message cache and use our own
    client
        .cache_and_http
        .cache
        .write()
        .settings_mut()
        .max_messages(0);

    {
        let mut data = client.data.write();
        let config = read_config();
        
        let db = DbInstance::new(&read_config().database.path, None)?;
        fetch_guild_config_from_db(&db)?;
    
        data.insert::<CustomEventList>(custom_events_arc);
        data.insert::<DatabaseKey>(Arc::new(db));
        data.insert::<InforKey>(Information::init(&client.cache_and_http.http)?);
        data.insert::<ReqwestClient>(Arc::new(Reqwest::new()));
        data.insert::<VoiceManager>(client.voice_manager.clone());
        data.insert::<CacheStorage>(Arc::new(MyCache::new()?));
        data.insert::<AIStore>(mutex_data(Eliza::from_file(&config.eliza_brain)?));
    
        if has_external_command("ffmpeg") {
            data.insert::<MusicManager>(mutex_data(HashMap::new()));
        }
    }

    client.with_framework(get_framework());

    let voices = client.voice_manager.clone();
    let cache = client.cache_and_http.cache.clone();
    let data = client.data.clone();
    let shard_manager = client.shard_manager.clone();

    ctrlc::set_handler(move || {
        info!("{}", "RECEIVED THE EXIT SIGNAL".red().bold().underlined());

        let cache_lock = cache.read();
        let guilds = cache_lock.all_guilds();
        let mut manager = voices.lock();

        info!("Disconnecting from {} guilds before exit", guilds.len());
        for guild in guilds {
            manager.leave(guild);
        }

        data.read().get::<CacheStorage>().unwrap().clean_up();

        info!("{}", "BYE".underlined().gradient(Color::Red));
        shard_manager.lock().shutdown_all();
    })?;

    client.start()?;

    println!("Bye");

    Ok(())
}

#[inline]
fn mutex_data<T>(data: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(data))
}

#[inline]
fn read_config() -> parking_lot::RwLockReadGuard<'static, Config> {
    global::CONFIG.read()
}

#[inline]
fn write_config() -> parking_lot::RwLockWriteGuard<'static, Config> {
    global::CONFIG.write()
}

fn fetch_guild_config_from_db(db: &DbInstance) -> Result<()> {
    let data = db.open("GuildConfig")?.get_all_json::<GuildConfig>()?;
    let guilds_config = &crate::read_config().guilds;

    for (k, v) in data {
        let key = bytes_to_le_u64(k).into();
        guilds_config.insert(key, v);
    }

    Ok(())
}
