use crate::commands::prelude::*;

#[command]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Watching configurated channels instead of all channel
/// This *will not* remove all channel nor disable the SadKaede-finder
/// only stop watching all channels if the `option sadkaede all` was enabled 
async fn disable_all(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };
    
    let config = crate::read_config().await;
    let description = match config.guilds.get_mut(&guild_id) {
        Some(ref mut g) if g.find_sadkaede.all => {
            g.find_sadkaede.all = false;
            update_guild_config(&ctx, &g).await?;
            "Ok, I'll not watching all the channel like before"
        }
        
        _ => "Nope"
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