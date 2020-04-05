use crate::commands::prelude::*;

#[command]
#[owners_only]
/// Reload the `config.toml`
async fn reload(ctx: &mut Context, msg: &Message) -> CommandResult {
    let db = get_data::<DatabaseKey>(&ctx).await.unwrap();
    let mut config = crate::write_config().await;
    tokio::task::spawn_blocking(move || config.reload(&db)).await??;
    
    msg.channel_id.say(ctx, "Reloaded the config").await?;
    Ok(())
}