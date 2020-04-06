use crate::commands::prelude::*;
use crate::types::GuildConfig;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Watching all channels for sauces
/// By enable this, all channels will be watched, ignoring the `add` and `remove` command
/// Disable this functionality by using `option sauce disable_all`
/// __**Note**__: this will enable the saucing machine automatically
async fn all(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let mut guild_config = config
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(&guild_id));
        
        
    let description = if guild_config.find_sauce.all {
        "The saucing machine is already enabled for all channels"
    } else {
        guild_config.find_sauce.all = true;
        guild_config.enable_find_sauce();
        update_guild_config(&ctx, &guild_config).await?;
        "Enabled the saucing machine for **ALL** channels"
    };
        
    let thumbnail = config.sauce.thumbnail.to_owned();
    let color = config.color.information;
    
    drop(guild_config);
    drop(config);
        
    msg.channel_id.send_message(&ctx, |m| m.embed(|embed| {
        embed.title("Saucing information");
        embed.thumbnail(thumbnail);
        embed.color(color);
        embed.timestamp(now());
        
        embed.description(description);
       
        embed
    })).await?;
    
    Ok(())
}