use crate::commands::prelude::*;
use crate::storages::ReqwestClient;
use crate::MaziiRequester;
use magic::traits::MagicStr;

#[command]
#[aliases("k")]
#[usage = "<Kanji(s)>"]
#[example = "智花"]
#[description = "Get the details meaning of kanji(s)"]
fn kanji(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.broadcast_typing(&ctx)?;
    let content = args.rest();
    let reqwest = get_data::<ReqwestClient>(&ctx).unwrap();
    let kanjis = block_on(reqwest.kanji(&content))?;

    let mut fields = Vec::new();

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
            .and_then(|d| d.split_at_limit(1980, "\n").next().map(String::from))
            .unwrap_or_default();

        fields.push((info, detail, false));
    }

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|embed| embed.color(0x977df2).fields(fields))
    })?;
    Ok(())
}
