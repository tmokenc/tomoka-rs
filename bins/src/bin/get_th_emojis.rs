use dotenv::dotenv;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;

type Emojis = HashMap<String, u64>;

struct Handler;
impl EventHandler for Handler {}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let id = 512404836306649098;
    let bot = env::var("DISCORD_TOKEN")?;

    let client = Client::new(bot, Handler)?;
    let guild = GuildId(id);
    dbg!("Logged in");
    let mut emojis: Emojis = HashMap::new();
    guild
        .to_partial_guild(&client.cache_and_http.http)?
        .emojis
        .values()
        .filter(|v| v.name.starts_with("th"))
        .for_each(|v| {
            emojis.insert(v.name.to_owned(), v.id.0);
        });

    let file = File::create("./tmq_emo.json")?;
    serde_json::to_writer_pretty(file, &emojis)?;

    Ok(())
}
