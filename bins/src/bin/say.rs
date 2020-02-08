use serenity::{
    model::{gateway::Ready, id::ChannelId},
    prelude::*,
};
use std::env;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

use dotenv::dotenv;

struct Handler {
    channel: u64,
    msg: String,
}

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        println!("Logged in as {}", ready.user.name);

        let channel_id = ChannelId(self.channel);

        channel_id.broadcast_typing(&ctx.http).unwrap();
        sleep(Duration::new(2, 500));

        println!("Sending the message: {}", &self.msg);
        if let Err(why) = channel_id.say(&ctx.http, self.msg.to_owned()) {
            eprintln!("Error while sending the message, error: {:#?}", why);
        }

        ctx.shard.shutdown_clean();
        println!("Logged out");
        std::process::exit(0);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let handler = {
        let mut arg = env::args().skip(1).collect::<Vec<String>>();
        let channel = match arg.get(0) {
            Some(c) => {
                if !c.chars().all(char::is_numeric) {
                    418811018698031107
                } else {
                    arg.remove(0).parse()?
                }
            }
            _ => 418811018698031107,
        };

        Handler {
            channel,
            msg: arg.join(" "),
        }
    };

    let token = env::var("DISCORD_TOKEN")?;
    let mut client = Client::new(&token, handler)?;

    client.start()?;

    Ok(())
}
