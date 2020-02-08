use crate::commands::prelude::*;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Watching configurated channels instead of all channel
/// This *will not* remove all channel nor disable the saucing machine,
/// only stop watching all channels if the `option checksauce all` was enabled 
fn disable_all(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let mut guild_config = crate::read_config()
        .guilds
        .get_mut(&guild_id);
        
    msg.channel_id.send_message(&ctx, |m| m.embed(|embed| {
        embed.title("Saucing information");
        embed.thumbnail("https://www.daringgourmet.com/wp-content/uploads/2017/04/Sweet-Sour-Sauce-1.jpg");
        embed.color(INFORMATION_COLOR);
        embed.timestamp(now());
        
        match guild_config {
            Some(ref mut g) if g.find_sauce.all => {
                g.find_sauce.all = false;
                update_guild_config(&ctx, &g).ok();
                embed.description("Ok, I'll not watching all the channel like before");
            }
            
            _ => {
                embed.description("Nope");
            }
        };
        
        embed
    }))?;
    
    Ok(())
}