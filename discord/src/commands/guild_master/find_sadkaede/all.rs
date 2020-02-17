use crate::commands::prelude::*;
use crate::types::GuildConfig;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Watching all channels for sauces
/// By enable this, all channels will be watched, ignoring the `add` and `remove` command
/// Disable this functionality by using `option sauce disable_all`
/// __**Note**__: this will enable the saucing machine automatically
fn all(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config();
    let mut guild_config = config
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(&guild_id));
        
    msg.channel_id.send_message(&ctx, |m| m.embed(|embed| {
        embed.title("SadKaede-finder information");
        embed.thumbnail(&config.sadkaede.thumbnail);
        embed.color(config.color.information);
        embed.timestamp(now());
        
        if guild_config.find_sadkaede.all {
            embed.description("The sadKaede-finder is already enabled for all channels");
        } else {
            guild_config.find_sadkaede.all = true;
            guild_config.enable_find_sadkaede();
            update_guild_config(&ctx, &guild_config).ok();
            embed.description("Enabled the sadkaede-finder for **ALL** channels");
        }
        
        embed
    }))?;
    
    Ok(())
}