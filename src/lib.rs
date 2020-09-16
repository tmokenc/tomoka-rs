#![allow(clippy::unreadable_literal)]

#[macro_use]
extern crate log;

extern crate config as lib_config;

mod cache;
mod commands;
mod config;
mod constants;
mod events;
mod framework;
mod global;
mod storages;
mod traits;
mod tui;
mod types;
mod utils;

pub mod logger;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub use requester::*;
pub use serenity::framework::standard::macros::hook;

use std::error::Error;
use std::sync::Arc;

use crate::config::Config;
use crate::logger::EventLogger;
use cache::MyCache;
use db::DbInstance;
use events::Handler;
use storages::*;
use types::*;

use colorful::{Color, Colorful};
use eliza::Eliza;
use futures::future;
use magic::dark_magic::has_external_command;
use serenity::client::bridge::gateway::GatewayIntents;
use serenity::model::id::GuildId;
use serenity::Client;
use tokio::signal::{self, unix};
use tokio::sync::Mutex;

pub async fn start_with_db(token: impl AsRef<str>, db: Arc<DbInstance>) -> Result<()> {
    let handler = Handler::new();
    let raw_handler = tomo_serenity_ext::MultiRawHandler::new();
    let raw_handler_clone = raw_handler.clone();
    let framework = framework::get_framework();
    
    raw_handler.add("Logger", EventLogger::new()).await;

    let mut client = Client::new(token.as_ref())
        .guild_subscriptions(false)
        .framework(framework)
        .event_handler(handler)
        .raw_event_handler(raw_handler_clone)
        .intents(intents())
        .await?;

    {
        let (mut data, config) = future::join(client.data.write(), read_config()).await;

        let req = Reqwest::new();
        fetch_guild_config_from_db(&db).await?;
        if let Err(why) = commands::pokemon::update_pokemon(&db, &req).await {
            error!("\n{}", why);
        }

        data.insert::<RawEventList>(raw_handler);
        data.insert::<DatabaseKey>(db);
        data.insert::<InforKey>(Information::init(&client.cache_and_http.http).await?);
        data.insert::<ReqwestClient>(Arc::new(req));
        data.insert::<CacheStorage>(Arc::new(MyCache::new(config.temp_dir.as_ref())?));
        data.insert::<AIStore>(mutex_data(Eliza::from_file(&config.eliza_brain).unwrap()));

        if has_external_command("ffmpeg") {
            data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
            // data.insert::<MusicManager>(mutex_data(HashMap::new()));
        }
    }

    let shard_manager = Arc::clone(&client.shard_manager);
    let mut term_sig = unix::signal(unix::SignalKind::terminate())?;
    tokio::spawn(async move {
        let mut sig = Box::pin(term_sig.recv());
        let ctrl_c = Box::pin(signal::ctrl_c());
        futures::future::select(sig.as_mut(), ctrl_c).await;
        info!("{}", "RECEIVED THE EXIT SIGNAL".red().bold().underlined());
        shard_manager.lock().await.shutdown_all().await;
    });

    client.start().await?;
    info!("{}", "BYE".underlined().gradient(Color::Red));
    Ok(())
}

pub async fn start(token: impl AsRef<str>) -> Result<()> {
    let db = db::get_db_instance(&read_config().await.database.path, None)
        .await
        .map(Arc::new)
        .ok_or("Cannot get the DbInstance")?;
    
    start_with_db(token, db).await
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
    let data = db.open("GuildConfig")?.get_all::<u64, GuildConfig>();
    let guilds_config = &crate::read_config().await.guilds;

    for (k, v) in data {
        guilds_config.insert(GuildId(k), v);
    }

    Ok(())
}

#[inline]
fn intents() -> GatewayIntents {
    GatewayIntents::all()
        // & !GatewayIntents::GUILD_MEMBERS
        & !GatewayIntents::GUILD_BANS
        & !GatewayIntents::GUILD_EMOJIS
        & !GatewayIntents::GUILD_INTEGRATIONS
        & !GatewayIntents::GUILD_WEBHOOKS
        & !GatewayIntents::GUILD_INVITES
        & !GatewayIntents::GUILD_PRESENCES
        & !GatewayIntents::GUILD_MESSAGE_TYPING
        & !GatewayIntents::DIRECT_MESSAGE_TYPING
}
