use dotenv::dotenv;
use serenity::model::id::UserId;
use serenity::prelude::*;
use std::env;
use std::error::Error;

struct Handler;
impl EventHandler for Handler {}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN")?;

    let mut args = env::args().skip(1).collect::<Vec<String>>();

    let id = env::var("ID").unwrap_or(args.remove(0)).parse::<u64>()?;
    let message = args.join(" ");

    let client = Client::new(&token, Handler)?;
    let http = &client.cache_and_http.http;

    let user = http.get_current_user()?;
    println!("Logged in as {}#{}", user.name, user.discriminator);

    let dm = UserId(id).create_dm_channel(http)?;

    println!("Sending the message: {}", message);
    dm.say(http, message)?;

    println!("Done");
    Ok(())
}
