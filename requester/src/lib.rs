#[macro_use]
extern crate async_trait;

pub mod ehentai;
pub mod mazii;
pub mod smogon;
pub mod urban;

pub use ehentai::EhentaiApi;
pub use mazii::MaziiRequester;
pub use smogon::SmogonRequester;
pub use urban::UrbanRequester;

pub use reqwest::get;
pub use reqwest::Client as Reqwest;
pub use reqwest::Error as ReqwestError;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;
