use crate::commands::prelude::*;
use std::process::Command;

const TRIGGLE_FILE: &str = "./.trigger";

#[command]
#[owners_only]
#[description = r#"Restart the bot, only works when the bot is running with the following bash script
```bash
cargo watch --no-gitignore -w .trigger -s "./target/release/tomoka_rs"
```"#]
fn restart(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    msg.channel_id
        .say(ctx, "I'll be right back in a second!\nPlease wait for me!")?;

    Command::new("bash").arg(TRIGGLE_FILE).output()?;
    Ok(())
}
