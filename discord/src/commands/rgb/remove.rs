use crate::commands::prelude::*;

#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_ROLES)]
#[description = "Remove roles from the almighty RGB database"]
fn remove(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    if msg.mention_roles.is_empty() {
        msg.channel_id
            .say(&ctx, "Please mention some role to be deleted")?;
        return Ok(());
    }

    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config();
    let mut guild = match config.guilds.get_mut(&guild_id) {
        Some(v) => v,
        None => {
            msg.channel_id.say(&ctx, "This guild hasn't been rgblized yet...")?;
            return Ok(())
        }
    };

    let roles = msg.mention_roles.iter().map(|v| v.0);
    let count = guild.remove_rgb(roles);

    let response = if count == 0 {
        "These roles aren't in the RGB list...".to_owned()
    } else {
        update_guild_config(&ctx, &guild)?;
        format!("Removed {} roles from the almighty RGB database", count)
    };

    msg.channel_id.say(&ctx, response)?;
    
    Ok(())
}