use crate::commands::prelude::*;

#[command]
#[only_in(guilds)]
#[owner_privilege]
#[required_permissions(MANAGE_ROLES)]
async fn add_member(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.mentions.is_empty() {
        return Err("Someone must be mentioned to be in the @RGB group".into())
    }
    
    let guild_id = msg.guild_id.ok_or("This must be use in guilds")?;
    
    let config = crate::read_config().await;
    let guilds = config.guilds.get(&guild_id);
    let roles = guilds
        .as_deref()
        .and_then(|v| v.rgblized.as_ref())
        .into_iter()
        .flatten()
        .map(|v| v.id)
        .collect::<Vec<_>>();
        
    drop(guilds);
    drop(config);
        
    for member in msg.mentions.iter() {
        let message = format!("Adding {} roles to <@{}>...", roles.len(), member.id.0);
        let mut mess = msg.channel_id.say(ctx, &message).await?;
        for role in roles.iter().copied() {
            if member.has_role(ctx, guild_id, role).await? {
                continue
            }
            
            ctx.http.add_member_role(guild_id.0, member.id.0, role).await?;
        }
        
        mess.edit(ctx, |m| m.content(message + " Done!")).await?;
    }
    
    Ok(())
}