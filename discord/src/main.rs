use dotenv::dotenv;
use std::env;
use tokio::runtime::Runtime;
use tomoka_rs::Result;

fn main() -> Result<()> {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN")?;
    tomoka_rs::logger::init()?;
    Runtime::new()?.block_on(tomoka_rs::start(token))
}
