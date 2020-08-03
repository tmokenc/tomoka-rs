use crate::commands::prelude::*;

#[command]
#[owners_only]
/// Save config to a file
async fn save_config(ctx: &Context, msg: &Message) -> CommandResult {
    let config = crate::read_config().await;
    
    let path = config
        .temp_dir
        .to_owned()
        .unwrap_or_else(|| ".".into());
        
    let file = config.save_file(path).await?;
    
    drop(config);
    
    msg.channel_id.send_message(ctx, |m| {
        m.content("Saved successfully!").add_file(&file)
    }).await?;
    
    Ok(())
}