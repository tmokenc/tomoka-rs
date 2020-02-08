use crate::config::Config;
use lazy_static::lazy_static;
use scheduled_thread_pool::ScheduledThreadPool;

lazy_static! {
    pub static ref CONFIG: Config = Config::from_env().unwrap();
    pub static ref GLOBAL_POOL: ScheduledThreadPool = ScheduledThreadPool::new(3);
}
