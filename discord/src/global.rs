use crate::config::Config;
use lazy_static::lazy_static;
use scheduled_thread_pool::ScheduledThreadPool;
use tokio::sync::RwLock;

lazy_static! {
    pub static ref CONFIG: RwLock<Config> = RwLock::new(Config::init().unwrap());
    pub static ref GLOBAL_POOL: ScheduledThreadPool = ScheduledThreadPool::new(3);
}
