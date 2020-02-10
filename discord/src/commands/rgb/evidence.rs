use crate::commands::prelude::*;
use log::error;
use rand::prelude::*;
use std::fs;

#[command]
#[aliases("evi")]
#[description = "The evidence of RGB"]
fn evidence(ctx: &mut Context, msg: &Message) -> CommandResult {
    let config = crate::read_config();
    let rgb = match config.rgb {
        Some(ref rgb) => rgb,
        None => {
            error!("No evidence directory has been set...");
            return Ok(());
        }
    };

    let evi = fs::read_dir(&rgb.evidence)?
        .filter_map(|v| v.ok())
        .choose(&mut SmallRng::from_entropy())
        .unwrap()
        .path();

    msg.channel_id.send_message(&ctx, |m| m.add_file(&evi))?;
    Ok(())
}
