use crate::commands::prelude::*;
use crate::storages::ReqwestClient;
use crate::MaziiRequester;
use magic::traits::MagicStr;

#[command]
#[aliases("k")]
#[usage = "<Kanji(s)>"]
#[example = "智花"]
/// Get the details meaning of kanji(s)
async fn kanji(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.broadcast_typing(&ctx).await?;
    let content = args.rest();
    let reqwest = get_data::<ReqwestClient>(&ctx).await.unwrap();
    let kanjis = reqwest.kanji(&content).await?;

    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.color(0x977df2);
        
        for kanji in kanjis {
            let info = format!(
                "{} - {level} {mean} | {on} {kun}",
                kanji.kanji,
                mean = kanji.mean,
                on = kanji.normal_on(),
                level = kanji.level.map(|l| format!("(N{})", l)).unwrap_or_default(),
                kun = kanji
                    .normal_kun()
                    .map(|k| format!("| {}", k))
                    .unwrap_or_default()
            );
    
            let detail = kanji
                .normal_detail()
                .and_then(|d| d.split_at_limit(1024, "\n").next().map(String::from))
                .unwrap_or_default();
    
            embed.field(info, detail, false);
        }
   
        embed
    })).await?;
    
    Ok(())
}
