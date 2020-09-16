use crate::cache::MyCache;
use crate::types::*;
use db::DbInstance;
use eliza::Eliza;
use requester::Reqwest;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use tokio::sync::Mutex;

type MutexData<T> = Arc<Mutex<T>>;

pub struct RawEventList;
impl TypeMapKey for RawEventList {
    type Value = tomo_serenity_ext::MultiRawHandler;
}

pub struct InforKey;
impl TypeMapKey for InforKey {
    type Value = Information;
}

pub struct ReqwestClient;
impl TypeMapKey for ReqwestClient {
    type Value = Arc<Reqwest>;
}

pub struct DatabaseKey;
impl TypeMapKey for DatabaseKey {
    type Value = Arc<DbInstance>;
}

pub struct VoiceManager;
impl TypeMapKey for VoiceManager {
    type Value = Arc<serenity::prelude::Mutex<ClientVoiceManager>>;
}

pub struct ReminderNotify;
impl TypeMapKey for ReminderNotify {
    type Value = Arc<tokio::sync::Notify>;
}

pub struct CacheStorage;
impl TypeMapKey for CacheStorage {
    type Value = Arc<MyCache>;
}

pub struct AIStore;
impl TypeMapKey for AIStore {
    type Value = MutexData<Eliza>;
}
