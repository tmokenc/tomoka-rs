use crate::commands::prelude::*;
use blocking::unblock;
use std::process::Command;
use crate::traits::{Embedable, CreateEmbed, ChannelExt};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Translated {
    text: String,
    from: String,
    to: String,
    did_you_mean: bool,
    auto_corrected: Option<String>,
}

impl Embedable for Translated {
    fn append(&self, embed: &mut CreateEmbed) {
        embed.title(format!("{} -> {}", self.from, self.to));
        embed.description(format!("=> {}", self.text));
        
        if self.did_you_mean {
            let text = format!("language: {}", self.from);
            embed.field("Did you mean", text, true);
        }
        
        if let Some(text) = &self.auto_corrected {
            embed.field("Autocorrected", text, true);
        }
    }
}


#[command]
#[aliases("trans", "t")]
/// Translate text using google translator
async fn translate(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let text = args.rest().to_owned();
    let process = unblock(move || {
        Command::new("node")
            .arg("./nodejs/translate/index.js")
            .arg(text)
            .output()
    }).await?;

    if !process.stderr.is_empty() {
        let err = String::from_utf8(process.stderr).unwrap();
        return Err(err.into())
    }
    
    let result: Translated = serde_json::from_slice(&process.stdout)?;
    msg.channel_id.send_embed(ctx).with_embedable_object(result).await?;
    Ok(())
}