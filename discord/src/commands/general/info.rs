use crate::commands::prelude::*;

#[command]
#[aliases("information")]
fn info(ctx: &mut Context, msg: &Message, _arg: Args) -> CommandResult {
    let data = ctx.data.read();
    let user_info = data.get::<InforKey>().unwrap();
    let my_info = ctx.http.get_current_application_info()?;

    let description = format!(
        "Hi, I'm {}\nCreated by: {}#{:04}\n{}",
        my_info.name, 
        my_info.owner.name, 
        my_info.owner.discriminator,
        my_info.description,
    );
    
    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|embed| {
            embed.description(description).field(
                "Executed commands",
                user_info.executed_commands(),
                true,
            )
        })
    })?;

    Ok(())
}
