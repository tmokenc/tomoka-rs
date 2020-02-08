use crate::commands::prelude::*;
use log::error;
use rand::prelude::*;
use std::fs;

#[command]
#[aliases("evi")]
#[description = "The evidence of RGB"]
fn evidence(ctx: &mut Context, msg: &Message) -> CommandResult {
    let path = match crate::read_config().rgb_evidence {
        Some(ref p) => p,
        None => {
            error!("No evidence directory has been set...");
            return Ok(());
        }
    };

    let evi = fs::read_dir(path)?
        .filter_map(|v| v.ok())
        .choose(&mut SmallRng::from_entropy())
        .unwrap()
        .path();

    msg.channel_id.send_message(&ctx, |m| m.add_file(&evi))?;
    Ok(())
}
