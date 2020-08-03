use crate::commands::prelude::*;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Disable the SadKaede-finder
async fn disable(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let description = match config.guilds.get_mut(&guild_id) {
        Some(ref mut g) if g.find_sadkaede.enable => {
            g.disable_find_sadkaede();
            update_guild_config(&ctx, &g).await?;
            "Disabled the sadkaede-finder"
        }
        
        _ => "The machine is already disabled"
    };
    
    let thumbnail = config.sadkaede.thumbnail.to_owned();
    let color = config.color.information;
    drop(config);
        
    msg.channel_id.send_message(&ctx, |m| m.embed(|embed| {
        embed.title("SadKaede-finder information");
        embed.thumbnail(thumbnail);
        embed.color(color);
        embed.timestamp(now());
        
        embed.description(description);
        
        embed
    })).await?;
    
    Ok(())
}