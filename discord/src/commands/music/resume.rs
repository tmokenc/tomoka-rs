use crate::commands::prelude::*;
use crate::storages::MusicManager;
use crate::types::PlayOption;
#[command]
fn resume(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(()),
    };

    let manager = ctx.data.read().get::<MusicManager>().cloned().unwrap();

    if let Some(music) = manager.lock().get(&guild_id) {
        music.resume();
    }

    Ok(())
}
