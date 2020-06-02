use crate::commands::prelude::*;
use crate::traits::{Embedable, Paginator};
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
    
    get_data::<ReqwestClient>(&ctx)
        .await
        .ok_or(magic::Error)?
        .google_image(text, sfw)
        .await?
        .into_iter()
        .map(ImageSearch::from)
        .collect::<Vec<_>>()
        .pagination(ctx, msg)
        .await?;
    
    Ok(())
}

impl Embedable for ImageSearch {
    fn append_to<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        embed.title(&self.title);
        embed.url(&self.url);
        embed.image(&self.image);
        embed
        
    }
}