use crate::commands::prelude::*;
use crate::traits::ToEmbed as _;
use requester::ehentai::EhentaiApi as _;

#[command]
#[min_args(1)]
/// Get the e-h gallery info
fn info(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.broadcast_typing(&ctx)?;

    let content = args.rest();
    let data = parse_eh_token(content);

    if data.is_empty() {
        msg.channel_id.send_message(ctx, |m| {
            m.embed(|embed| {
                embed.title("SadKaede information");
                embed.description("Error 404 Not found SadKaede in the content...");

                {
                    let config = crate::read_config();
                    embed.color(config.color.error);
                    embed.thumbnail(&config.sadkaede.thumbnail);
                }

                embed
            })
        })?;

        return Ok(());
    };

    let reqwest = get_data::<ReqwestClient>(&ctx).unwrap();
    let nsfw = is_nsfw_channel(&ctx, msg.channel_id);
    let data = block_on(reqwest.gmetadata(data))?
        .into_iter()
        .filter(|v| nsfw || v.is_sfw())
        .collect::<Vec<_>>();

    if data.is_empty() {
        msg.channel_id.send_message(ctx, |m| {
            m.embed(|embed| {
                embed.title("SadKaede information");
                embed.description("Succesfully Not Found");

                {
                    let config = crate::read_config();
                    embed.color(config.color.error);
                    embed.thumbnail(&config.sadkaede.thumbnail);
                }

                embed
            })
        })?;

        return Ok(());
    }

    for d in data {
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|mut embed| {
                d.to_embed(&mut embed);
                embed
            })
        })?;
    }

    Ok(())
}
