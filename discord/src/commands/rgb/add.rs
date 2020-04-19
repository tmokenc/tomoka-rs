use crate::commands::prelude::*;
use crate::types::GuildConfig;

#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_ROLES)]
/// Add roles to the almighty RGB databse
async fn add(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    if msg.mention_roles.is_empty() {
        msg.channel_id
            .say(&ctx, "Please mention some role to be added").await?;
        return Ok(());
    }

    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let mut guild = config
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(guild_id.0));

    let mut roles = Vec::new();
    
    for role in &msg.mention_roles {
        if let Some(role) = role.to_role_cached(&ctx).await {
            roles.push(role);
        }
    }
    
    let count = guild.add_rgb(roles);

    let response = if count == 0 {
        "Please make sure that you are adding a mentionable roles, which is not in the rgb list yet"
            .to_owned()
    } else {
        update_guild_config(&ctx, &guild).await?;
        format!("Added {} roles into the almighty RGB database", count)
    };
    
    drop(guild);
    drop(config);

    msg.channel_id.say(&ctx, response).await?;
    Ok(())
}
