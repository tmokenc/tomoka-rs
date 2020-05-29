use super::Embedable;
use serenity::builder::CreateEmbed;

pub trait Paginator {
    /// Notice that the page start at 1
    fn append_page_data<'a>(&self, page: usize, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed;
    fn total_pages(&self) -> Option<usize> {
        None
    }
}

impl<E: Embedable> Paginator for Vec<E> {
    fn append_page_data<'a>(&self, page: usize, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        match self.get(page - 1) {
            Some(data) => data.append_to(embed),
            None => embed.description("This page does not exist")
        }
    }
    
    #[inline]
    fn total_pages(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl Paginator for requester::nhentai::NhentaiGallery {
    fn append_page_data<'a>(&self, page: usize, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        self.append_to(embed);
        
        let total_pages = self.total_pages();
        embed.footer(|f| {
            f.text(format!("ID: {} | Page: {} / {}", self.id, page, total_pages))
        });
        
        match self.page(page) {
            Some(p) => embed.image(p),
            None => embed.field("Error", format!("Out of page, this gallery has only {} pages", total_pages), false)
        };
        
        embed
    }
    
    fn total_pages(&self) -> Option<usize> {
        Some(self.total_pages())
    }
}