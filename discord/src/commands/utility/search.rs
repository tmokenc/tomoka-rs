use crate::commands::prelude::*;
use requester::google::{GoogleSearchData, GoogleScraper as _};

use serenity::builder::CreateEmbed;
use crate::traits::Paginator;

#[command]
#[aliases("g")]
#[min_args(1)]
/// Search gooogle
async fn search(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let text = args.rest().to_owned();
    
    let data = get_data::<ReqwestClient>(&ctx)
        .await
        .unwrap()
        .google_search(&text)
        .await?;
    let color = crate::read_config().await.color.information;
    
    (Search { data, text, color }).pagination(ctx, msg).await?;
    
    Ok(())
}

struct Search {
    data: Vec<GoogleSearchData>,
    text: String,
    color: u64,
}

impl Paginator for Search {
    fn append_page_data<'a>(
        &mut self, 
        page: core::num::NonZeroUsize,
        embed: &'a mut CreateEmbed
    ) -> &'a mut CreateEmbed {
        let data = &self.data[page.get() - 1];
        let description = format!("{}\n[[Link]]({})", data.description, data.link);
        embed.title(format!("Result for `{}`", self.text));
        embed.field(&data.title, description, false);
        embed.color(self.color);
        embed.footer(|f| f.text(format!("Result {} / {}", page, self.data.len())));
        
        embed
    }
    
    fn total_pages(&self) -> Option<usize> {
        Some(self.data.len())
    }
}