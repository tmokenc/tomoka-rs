use crate::commands::prelude::*;
use crate::types::PlayOption;

#[command]
fn stop(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(()),
    };

    let manager = ctx.data.read().get::<MusicManager>().cloned().unwrap();

    if let Some(music) = manager.lock().remove(&guild_id) {
        music.stop();
        ctx.data
            .read()
            .get::<VoiceManager>()
            .cloned()
            .unwrap()
            .lock()
            .leave(guild_id);
    }

    //if manager.lock().is_empty() {

    //}

    Ok(())
}
