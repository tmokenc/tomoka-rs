use crate::cache::MyCache;
use crate::events::RawEvents;
use crate::types::*;
use db::DbInstance;
use eliza::Eliza;
use requester::Reqwest;
// use serenity::model::id::GuildId;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::prelude::TypeMapKey;
// use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

type MutexData<T> = Arc<Mutex<T>>;

pub struct CustomEventList;
impl TypeMapKey for CustomEventList {
    type Value = Arc<RawEvents>;
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

// pub struct MusicManager;
// impl TypeMapKey for MusicManager {
//     type Value = MutexData<HashMap<GuildId, Music>>;
// }

pub struct CacheStorage;
impl TypeMapKey for CacheStorage {
    type Value = Arc<MyCache>;
}

pub struct AIStore;
impl TypeMapKey for AIStore {
    type Value = MutexData<Eliza>;
}
