use crate::commands::prelude::*;

#[command]
#[owners_only]
/// Reload the `config.toml`
async fn reload(ctx: &Context, msg: &Message) -> CommandResult {
    let db = get_data::<DatabaseKey>(&ctx).await.unwrap();
    let mut config = crate::write_config().await;
    
    config.reload(&db)?;
    
    drop(db);
    drop(config);
    
    msg.channel_id.say(ctx, "Reloaded the config").await?;
    Ok(())
}