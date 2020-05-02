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
mod storages;
mod traits;
mod types;
mod utils;

pub mod logger;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub use requester::*;
pub use serenity::framework::standard::macros::hook;

use std::error::Error;
use std::sync::Arc;

use crate::config::Config;
use cache::MyCache;
use db::DbInstance;
use events::{Handler, RawHandler};
use storages::*;
use types::*;

use colorful::{Color, Colorful};
use eliza::Eliza;
use futures::future;
use magic::dark_magic::has_external_command;
use serenity::model::id::GuildId;
use serenity::Client;
use serenity::client::bridge::gateway::GatewayIntents;
use tokio::signal::{self, unix};
use tokio::sync::Mutex;

pub async fn start(token: impl AsRef<str>) -> Result<()> {
    let handler = Handler::new();
    let raw_handler = RawHandler::new();
    let custom_events_arc = raw_handler.handler.clone();
    let framework = framework::get_framework();

    let mut client = Client::new(token.as_ref())
        .guild_subscriptions(false)
        .framework(framework)
        .event_handler(handler)
        .raw_event_handler(raw_handler)
        .intents(intents())
        .await?;

    // Disable the default message cache and use our own. Just in case
    client
        .cache_and_http
        .cache
        .write()
        .await
        .settings_mut()
        .max_messages(0);

    {
        let (mut data, config) = future::join(client.data.write(), read_config()).await;

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

    let shard_manager = Arc::clone(&client.shard_manager);
    let mut term_sig = unix::signal(unix::SignalKind::terminate())?;
    tokio::spawn(async move {
        let mut sig = Box::pin(term_sig.recv());
        let ctrl_c = Box::pin(signal::ctrl_c());
        futures::future::select(sig.as_mut(), ctrl_c).await;
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