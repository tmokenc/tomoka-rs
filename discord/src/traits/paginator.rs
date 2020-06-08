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
use std::env;
use core::str::FromStr;
use magic::types::Void;

#[derive(Debug, Clone)]
pub struct PaginatorReactions {
    reactions: [ReactionType; 5]
}

#[derive(Debug, Clone, Copy)]
pub enum PaginatorAction {
    Page(usize),
    First,
    Previous,
    Next,
    Last,
    Destroy,
}

impl FromStr for PaginatorAction {
    type Err = Void;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.to_lowercase();
        
        let res = match s.trim() {
            "first" => Self::First,
            "previous" => Self::Previous,
            "next" => Self::Next,
            "last" => Self::Last,
            s if s.starts_with("page") => Self::Page(s[4..].trim().parse()?),
            _ => return Err(Void),
        };
        
        Ok(res)
    }

}

impl Default for PaginatorReactions {
    #[rustfmt::skip]
    fn default() -> Self {
        let reactions = if let Ok(reactions) = env::var("PAGINATOR_REACTIONS") {
            use ReactionType::Unicode;
            let mut iter = reactions.split(',').map(ReactionType::try_from);
                
            [
                iter.next().and_then(|v| v.ok()).unwrap_or_else(|| Unicode(String::from("⏮️"))),
                iter.next().and_then(|v| v.ok()).unwrap_or_else(|| Unicode(String::from("◀️"))),
                iter.next().and_then(|v| v.ok()).unwrap_or_else(|| Unicode(String::from("▶️"))),
                iter.next().and_then(|v| v.ok()).unwrap_or_else(|| Unicode(String::from("⏭️"))),
                iter.next().and_then(|v| v.ok()).unwrap_or_else(|| Unicode(String::from("❌"))),
            ]
        } else {
            [
                ReactionType::Unicode(String::from("⏮️")),
                ReactionType::Unicode(String::from("◀️")),
                ReactionType::Unicode(String::from("▶️")),
                ReactionType::Unicode(String::from("⏭️")),
                ReactionType::Unicode(String::from("❌")),
            ]
        };
        
        Self { reactions }
    }
}

impl PaginatorReactions {
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=&ReactionType> {
        self.reactions.iter()
    }
    
    #[allow(dead_code)]
    pub fn change(&mut self, kind: PaginatorAction, reaction: ReactionType) {
        use PaginatorAction::*;
        
        let current = match kind {
            First => &mut self.reactions[0],
            Previous => &mut self.reactions[1],
            Next => &mut self.reactions[2],
            Last => &mut self.reactions[3],
            Destroy => &mut self.reactions[4],
            _ => return,
        };
        
        *current = reaction;
    }
    
    #[inline]
    pub fn kind(&self, reaction: &ReactionType) -> Option<PaginatorAction> {
        use PaginatorAction::*;
        
        self.iter().position(|ref v| v == &reaction).and_then(|v| match v {
            0 => Some(First),
            1 => Some(Previous),
            2 => Some(Next),
            3 => Some(Last),
            4 => Some(Destroy),
            _ => None
        })
    }
}

#[async_trait]
pub trait Paginator {
    /// Notice that the page start at 1
    fn append_page_data<'a>(&self, page: usize, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed;
    
    #[inline]
    fn default_page(&self) -> usize {
        1
    }
    
    #[inline]
    fn total_pages(&self) -> Option<usize> {
        None
    }
    
    #[inline]
    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
    
    #[inline]
    fn reactions(&self) -> PaginatorReactions {
        Default::default()
    }
    
    async fn pagination<C: Borrow<Context> + Send>(&self, ctx: C, msg: &Message) -> Result<()> {
        let ctx = ctx.borrow();
        let total = self.total_pages();
        let wait_time = self.timeout();
        let reactions = self.reactions();
        let mut current_page = self.default_page();
        let mut mess = msg
            .channel_id
            .send_message(ctx, |message| {
                message.reactions(reactions.iter().cloned());
                message.embed(|embed| self.append_page_data(current_page, embed));
                message
            })
            .await?;
    
        let react_collector = mess
            .await_reactions(ctx)
            .author_id(msg.author.id)
            .await
            .filter_map(|reaction| {
                let reaction = reaction.as_inner_ref().to_owned();
                let result = reactions.kind(&reaction.emoji);
                
                async move {
                    if result.is_some() {
                        let http = Arc::clone(&ctx.http);
                        tokio::spawn(async move {
                            reaction.delete(http).await.ok();
                        });
                    }
                
                    result
                }
            });
            
        let msg_collector = msg
            .channel_id
            .await_replies(ctx)
            .author_id(msg.author.id)
            .await
            .filter_map(|v| async move {
                v.content.parse::<PaginatorAction>().ok()
            });
            
        let stream = futures::stream::select(react_collector, msg_collector);
        futures::pin_mut!(stream);
    
        while let Ok(Some(action)) = timeout(wait_time, stream.next()).await {
            match action {
                PaginatorAction::First if current_page > 1 => current_page = 1,
                
                PaginatorAction::Previous if current_page > 1 => current_page -= 1,
                
                PaginatorAction::Next => match total {
                    Some(max) if current_page < max => current_page += 1,
                    _ => continue,
                }
                
                PaginatorAction::Last => match total {
                    Some(max) => current_page = max,
                    None => continue
                }
                
                PaginatorAction::Page(page) if current_page != page => match total {
                    Some(max) if page <= max => current_page = page,
                    _ => current_page = page,
                }
                
                PaginatorAction::Destroy => {
                    mess.delete(ctx).await?;
                    return Ok(());
                }
                
                _ => continue,
            }
    
            mess.edit(ctx, |m| m.embed(|e| self.append_page_data(current_page, e))).await?;
        }
    
        drop(stream);
    
        let futs = reactions
            .iter()
            .cloned()
            .map(|s| msg.channel_id.delete_reaction(ctx, mess.id.0, None, s));
    
        futures::future::join_all(futs).await;
        Ok(())
    }
}

impl<E: Embedable> Paginator for Vec<E> {
    fn append_page_data<'a>(&self, page: usize, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        embed.footer(|f| f.text(format!("{} / {}", page, self.len())));
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