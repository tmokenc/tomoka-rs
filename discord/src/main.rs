use dotenv::dotenv;
use std::env;
use tomoka_rs::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    let token = env::var("DISCORD_TOKEN")?;
    tomoka_rs::start(token).await?;
    Ok(())
}
