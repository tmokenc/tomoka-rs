use super::Embedable;
use serenity::builder::CreateEmbed;
use core::borrow::Borrow;
use core::convert::TryFrom;
use core::time::Duration;
use tokio::time::timeout;
use futures::stream::StreamExt;
use std::sync::Arc;
use serenity::client::Context;
use serenity::model::channel::{Message, ReactionType};
use async_trait::async_trait;
use crate::Result;


#[async_trait]
pub trait Paginator {
    /// Notice that the page start at 1
    fn append_page_data<'a>(&self, page: usize, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed;
    fn total_pages(&self) -> Option<usize> {
        None
    }
    
    async fn pagination<C: Borrow<Context> + Send>(&self, ctx: C, msg: &Message) -> Result<()> {
        const REACTIONS: &[&str] = &["◀️", "▶️", "❌"];
        const WAIT_TIME: Duration = Duration::from_secs(30);
    
        let ctx = ctx.borrow();
        let total = self.total_pages();
        let mut current_page = 1;
        let mut mess = msg
            .channel_id
            .send_message(ctx, |message| {
                let reactions = REACTIONS
                    .iter()
                    .map(|&s| ReactionType::try_from(s).unwrap());
    
                message.reactions(reactions);
                message.embed(|embed| self.append_page_data(current_page, embed));
                message
            })
            .await?;
    
        let mut collector = mess
            .await_reactions(&ctx)
            .author_id(msg.author.id)
            .filter(|reaction| {
                matches!(&reaction.emoji, ReactionType::Unicode(s) if REACTIONS.contains(&s.as_str()))
            })
            .await;
    
        while let Ok(Some(reaction)) = timeout(WAIT_TIME, collector.next()).await {
            let reaction = reaction.as_inner_ref();
    
            let http = Arc::clone(&ctx.http);
            let react = reaction.to_owned();
            tokio::spawn(async move {
                react.delete(http).await.ok();
            });
    
            let emoji = match &reaction.emoji {
                ReactionType::Unicode(s) => s,
                _ => continue,
            };
    
            match emoji.as_str() {
                "◀️" => {
                    if current_page == 1 {
                        continue;
                    }
    
                    current_page -= 1;
                }
    
                "▶️" => {
                    if matches!(total, Some(max) if current_page >= max) {
                        continue;
                    }
    
                    current_page += 1;
                }
    
                "❌" => {
                    mess.delete(ctx).await?;
                    return Ok(());
                }
                _ => continue,
            }
    
            mess.edit(ctx, |m| m.embed(|e| self.append_page_data(current_page, e))).await?;
        }
    
        drop(collector);
    
        let futs = REACTIONS
            .into_iter()
            .map(|&s| ReactionType::try_from(s).unwrap())
            .map(|s| msg.channel_id.delete_reaction(&ctx, mess.id.0, None, s));
    
        futures::future::join_all(futs).await;
        Ok(())
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
        let total_pages = self.total_pages();
        let color = {
            let num_length = (self.id as f32 + 1.0).log10().ceil() as u64;
            self.media_id * num_length + self.id
        };
        
        embed.title(&self.title.pretty);
        embed.url(self.url());
        embed.color(color);
        
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