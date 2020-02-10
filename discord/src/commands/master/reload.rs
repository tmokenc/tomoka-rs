use crate::commands::prelude::*;

#[command]
#[owners_only]
/// Reload the `config.toml`
fn reload(ctx: &mut Context, msg: &Message) -> CommandResult {
    let db = get_data::<DatabaseKey>(&ctx).unwrap();
    let mut config = crate::write_config();
    config.reload(&db)?;
    
    drop(config);
    
    msg.channel_id.say(ctx, "Reloaded the config")?;
    Ok(())
}