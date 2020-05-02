use crate::commands::prelude::*;
use std::process::Command;

const TRIGGLE_FILE: &str = "./.trigger";

#[command]
#[owners_only]
/// Restart the bot, only works when the bot is running with the following bash script
/// ```bash
/// cargo watch --no-gitignore -w .trigger -s "./target/release/tomoka_rs"
/// ```
async fn restart(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(ctx, "I'll be right back in a second!\nPlease wait for me!").await?;

    tokio::task::spawn_blocking(|| {
        Command::new("bash").arg(TRIGGLE_FILE).output()
    }).await??;
    
    Ok(())
}
