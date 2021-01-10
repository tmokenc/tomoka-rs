use serenity::framework::standard::macros::group;
use crate::commands::prelude::*;
use crate::traits::Embedable;
use crate::genshin;

#[group]
#[prefixes("genshin", "gi", "paimon")]
#[commands(timer)]
#[default_command(paimon)]
/// Genshin Impact utilities
struct Genshin;

#[command]
// #[permission(MANAGE_CHANNEL)]
/// Timer for genshin impact related events
async fn timer(ctx: &Context, msg: &Message) -> CommandResult {
    let mess = genshin::RegionsEmbed.send_embed(&ctx, msg.channel_id).await?;
    let key = msg.channel_id.as_u64();
    let val = mess.id.as_u64();
    
    get_data::<DatabaseKey>(&ctx)
        .await
        .unwrap()
        .open("genshin_watch")?
        .insert(key, val)?;
        
    Ok(())
}

#[command]
async fn paimon(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}

#[command]
async fn my_list(_ctx: &Context, _msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
async fn today(_ctx: &Context, _msg: &Message) -> CommandResult {
    Ok(())
}