use crate::commands::prelude::*;
use crate::traits::ToEmbed as _;
use requester::ehentai::EhentaiApi as _;
use serenity::model::id::ChannelId;
use crate::Result;

#[command]
#[min_args(1)]
/// Get the e-h gallery info
async fn info(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let content = args.rest();
    let data = parse_eh_token(content);

    if data.is_empty() {
        send_error(&ctx, msg.channel_id, "Error 404 Not found SadKaede in the content...").await?;
        return Ok(());
    };

    let nsfw = is_nsfw_channel(&ctx, msg.channel_id).await;
    let data = get_data::<ReqwestClient>(&ctx)
        .await
        .unwrap()
        .gmetadata(data)
        .await?
        .into_iter()
        .filter(|v| nsfw || v.is_sfw())
        .collect::<Vec<_>>();

    if data.is_empty() {
        send_error(&ctx, msg.channel_id, "Succesfully Not Found").await?;
        return Ok(());
    }

    for d in data {
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|mut embed| {
                d.to_embed(&mut embed);
                embed
            })
        }).await?;
    }

    Ok(())
}

async fn send_error(ctx: &Context, channel_id: ChannelId, msg: &str) -> Result<()> {
    let config = crate::read_config().await;
    let color = config.color.error;
    let thumbnail = config.sadkaede.thumbnail.to_owned();
    drop(config);
        
    channel_id.send_message(ctx, |m| {
        m.embed(|embed| {
            embed.title("SadKaede information");
            embed.description(msg);

            embed.color(color);
            embed.thumbnail(thumbnail);

            embed
        })
    }).await?;

    Ok(())
}