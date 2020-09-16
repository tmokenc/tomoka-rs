use crate::commands::prelude::*;
use tokio::process::Command;

const ERROR_MESSAGE = "Cannot find any code in your message.
Please make sure you wrap them inside the \\`\\`\\`your code\\`\\`\`";

#[command]
async fn execute(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    Ok(())
}

enum Language {
    Javascript,
    Rust,
    Unsupported
}

fn match_code(s: &str) -> Option<(&str, Language)> {
    let code = args.rest().split("```").nth(1)?;
    
    let data = if code.starts_with("js") || code.starts_with("javascript") {
        let code = code.trim_start("js").trim_start("javascript").trim();
        (code, Language::Javascript)
    } else if code.starts_with("rs") || code.starts_with("rust") {
        let code = code.trim_start("rs").trim_start("rust").trim();
        (code, Language::Rust)
    } else {
        (code, Language::Unsupported)
    };
    
    Some(data)
}

#[command]
#[aliases("js")]
async fn javascript(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let file = todo!();
    
    let code = match_code(s)
        .map(|(code, _)| code)
        .ok_or_else(|| ERROR_MESSAGE.into())?;
    
    let mut process = Command::new("deno")
        .arg("run")
        .arg(file_path)
        .spawn();
    
    Ok(())
}

#[command]
#[aliases("rs")]
async fn rust(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let code = match_code(s)
        .map(|(code, _)| code)
        .ok_or_else(|| ERROR_MESSAGE.into())?;
    
    Ok(())
}