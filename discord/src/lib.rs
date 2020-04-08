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

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub use requester::*;
pub use serenity::framework::standard::macros::hook;

use std::error::Error;
use std::sync::Arc;

use crate::config::Config;
use cache::MyCache;
use db::DbInstance;
use events::{Handler, RawHandler};
use magic::dark_magic::{bytes_to_le_u64, has_external_command};
use serenity::Client;
use storages::*;
use tokio::sync::Mutex;
use types::*;

use colorful::{Color, Colorful};
use eliza::Eliza;

pub async fn start(token: impl AsRef<str>) -> Result<()> {
    logger::init();

    let handler = Handler::new();
    let raw_handler = RawHandler::new();
    let custom_events_arc = raw_handler.handler.clone();

    info!(
        "Login with the token:\n{}",
        token.as_ref().to_owned().underlined().yellow()
    );
    let mut client = Client::new_with_extras(token.as_ref(), |event| {
        event
            .event_handler(handler)
            .raw_event_handler(raw_handler)
            .framework(framework::get_framework())
    })
    .await?;

    // Disable the default message cache and use our own
    client
        .cache_and_http
        .cache
        .write()
        .await
        .settings_mut()
        .max_messages(0);

    {
        let mut data = client.data.write().await;
        let config = read_config().await;

        let db = DbInstance::new(&config.database.path, None)?;
        fetch_guild_config_from_db(&db).await?;

        data.insert::<CustomEventList>(custom_events_arc);
        data.insert::<DatabaseKey>(Arc::new(db));
        data.insert::<InforKey>(Information::init(&client.cache_and_http.http).await?);
        data.insert::<ReqwestClient>(Arc::new(Reqwest::new()));
        data.insert::<CacheStorage>(Arc::new(MyCache::new(config.temp_dir.as_ref())?));
        data.insert::<AIStore>(mutex_data(Eliza::from_file(&config.eliza_brain).unwrap()));

        if has_external_command("ffmpeg") {
            data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
            // data.insert::<MusicManager>(mutex_data(HashMap::new()));
        }
    }

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("{}", "RECEIVED THE EXIT SIGNAL".red().bold().underlined());
        shard_manager.lock().await.shutdown_all().await;
        info!("{}", "BYE".underlined().gradient(Color::Red));
    });

    client.start().await?;
    Ok(())
}

#[inline]
fn mutex_data<T>(data: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(data))
}

#[inline]
async fn read_config() -> tokio::sync::RwLockReadGuard<'static, Config> {
    global::CONFIG.read().await
}

#[inline]
async fn write_config() -> tokio::sync::RwLockWriteGuard<'static, Config> {
    global::CONFIG.write().await
}

async fn fetch_guild_config_from_db(db: &DbInstance) -> Result<()> {
    let data = db.open("GuildConfig")?.get_all_json::<GuildConfig>()?;
    let guilds_config = &crate::read_config().await.guilds;

    for (k, v) in data {
        let key = bytes_to_le_u64(k).into();
        guilds_config.insert(key, v);
    }

    Ok(())
}
