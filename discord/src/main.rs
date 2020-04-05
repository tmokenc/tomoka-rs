use dotenv::dotenv;
use tomoka_rs::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    let token = std::env::var("DISCORD_TOKEN")?;
    tomoka_rs::start(token).await?;
    Ok(())
}
