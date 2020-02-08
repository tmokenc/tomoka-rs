use crate::commands::prelude::*;

#[command]
#[owners_only]
/// Save config to a file
fn save_config(ctx: &mut Context, msg: &Message) -> CommandResult {
    let file = crate::read_config().save_file("./temp")?;
    
    msg.channel_id.send_message(ctx, |m| {
        m.content("Saved successfully!").add_file(&file)
    })?;
    
    Ok(())
}