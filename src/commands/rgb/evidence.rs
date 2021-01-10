use crate::commands::prelude::*;
use log::error;
use rand::prelude::*;
use futures::stream::StreamExt;

#[command]
#[aliases("evi")]
/// The evidence of RGB
async fn evidence(ctx: &Context, msg: &Message) -> CommandResult {
    let config = crate::read_config().await;
    let rgb = match config.rgb {
        Some(ref rgb) => rgb,
        None => {
            error!("No evidence directory has been set...");
            return Ok(());
        }
    };

    let evi = fs::read_dir(&rgb.evidence)
        .await?
        .filter_map(|v| async {v.ok()})
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .choose(&mut SmallRng::from_entropy())
        .map(|v| v.path());

    if let Some(e) = evi {
        msg.channel_id.send_message(&ctx, |m| m.add_file(&e)).await?;
    }
    
    Ok(())
}
