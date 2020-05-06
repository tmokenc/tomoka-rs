use crate::commands::prelude::*;
use requester::GoogleScraper as _;
// use requester::DuckDuckGoScraper as _;
use requester::google::GoogleImageData;
use requester::duckduckgo::DuckDuckGoImageData;
use serenity::builder::CreateEmbed;

struct ImageSearch {
    title: String,
    url: String,
    image: String,
}

impl From<GoogleImageData> for ImageSearch {
    fn from(v: GoogleImageData) -> Self {
        Self {
            title: v.title,
            url: v.url,
            image: v.img_url,
        }
    }
}

impl From<DuckDuckGoImageData> for ImageSearch {
    fn from(v: DuckDuckGoImageData) -> Self {
        Self {
            title: v.title,
            url: v.url,
            image: v.image,
        }
    }
}

#[command]
#[aliases("img", "image")]
async fn search_image(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let text = args.rest();
    let sfw = !is_nsfw_channel(&ctx, msg.channel_id).await;
    let data = get_data::<ReqwestClient>(&ctx)
        .await
        .ok_or(magic::Error)?
        .google_image(text, sfw)
        //.duck_image(text, 0, true)
        .await?
        .into_iter()
        .map(|v| ImageSearch::from(v))
        .collect::<Vec<_>>();
    
    paginator(ctx, msg, data, embed_data).await?;
    Ok(())
}

fn embed_data<'a>(embed: &'a mut CreateEmbed, data: &ImageSearch) -> &'a mut CreateEmbed {
    embed.title(&data.title);
    embed.url(&data.url);
    embed.image(&data.image);
    embed
}