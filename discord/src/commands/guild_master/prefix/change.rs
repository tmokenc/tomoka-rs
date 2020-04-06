use crate::commands::prelude::*;
use crate::types::GuildConfig;

#[command]
#[aliases("set")]
#[bucket = "basic"]
#[only_in(guilds)]
#[min_args(1)]
#[required_permissions(MANAGE_GUILD)]
///Set a custom prefix instead of the default
async fn change(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let prefix = args.rest();
    if prefix.is_empty() {
        return Ok(());
    }
    
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(())
    };

    let config = crate::read_config().await;
    let mut g = config
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(guild_id.0));

    let old_prefix = g.set_prefix(&prefix);
    let color = config.color.information;
    
    update_guild_config(&ctx, &g).await?;
    drop(g);
    drop(config);
    
    let description = format!("Changed the current prefix to **__{}__**", &prefix);
    
    msg.channel_id.send_message(&ctx, |m| m.embed(|embed| {
        embed.title("Prefix information");
        embed.color(color);
        embed.timestamp(now());
        
        embed.description(description);
        embed.field("New prefix", prefix, true);
        
        if let Some(prefix) = old_prefix {
            embed.field("Old prefix", prefix, true);
        }
        
        embed
    })).await?;

    Ok(())
}