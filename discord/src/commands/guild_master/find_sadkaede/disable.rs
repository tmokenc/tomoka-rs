use crate::commands::prelude::*;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Disable the SadKaede-finder
fn disable(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config();
    let mut guild_config = config
        .guilds
        .get_mut(&guild_id);
        
    msg.channel_id.send_message(&ctx, |m| m.embed(|embed| {
        embed.title("SadKaede-finder information");
        embed.thumbnail(&config.sadkaede.thumbnail);
        embed.color(config.color.information);
        embed.timestamp(now());
        
        match guild_config {
            Some(ref mut g) if g.find_sadkaede.enable => {
                g.disable_find_sadkaede();
                update_guild_config(&ctx, &g).ok();
                embed.description("Disabled the sadkaede-finder");
            }
            
            _ => {
                embed.description("The machine is already disabled");
            }
        };
        
        embed
    }))?;
    
    Ok(())
}