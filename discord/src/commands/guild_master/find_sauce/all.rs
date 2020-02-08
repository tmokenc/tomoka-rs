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
    
    let mut guild_config = crate::read_config()
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(&guild_id));
        
    msg.channel_id.send_message(&ctx, |m| m.embed(|embed| {
        embed.title("Saucing information");
        embed.thumbnail("https://www.daringgourmet.com/wp-content/uploads/2017/04/Sweet-Sour-Sauce-1.jpg");
        embed.color(INFORMATION_COLOR);
        embed.timestamp(now());
        
        if guild_config.find_sauce.all {
            embed.description("The saucing machine is already enabled for all channels");
        } else {
            guild_config.find_sauce.all = true;
            guild_config.enable_find_sauce();
            update_guild_config(&ctx, &guild_config).ok();
            embed.description("Enabled the saucing machine for **ALL** channels");
        }
        
        embed
    }))?;
    
    Ok(())
}