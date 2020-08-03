use crate::config::Config;
use lazy_static::lazy_static;
use tokio::sync::RwLock;

lazy_static! {
    pub static ref CONFIG: RwLock<Config> = RwLock::new(Config::init().unwrap());
}
