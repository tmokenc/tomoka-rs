use crate::Result;
use serenity::model::id::ChannelId;
use serenity::builder::CreateEmbed;
use serenity::http::client::Http;
use serenity::model::channel::{ReactionType, Message};
use serenity::http::AttachmentType;
use serde_json::{json, Value};
use serde_json::map::Map;
use magic::traits::{MagicStr, MagicOption};
use core::future::Future;
use core::convert::TryFrom;
use futures::ready;
use std::pin::Pin;
use std::task::{Poll, Context};

type BoxedFuture<'a, T> = Pin<Box<dyn core::future::Future<Output = T> + 'a + Send>>;
pub struct SendMessageBuilder<'a> {
    http: Option<Box<dyn AsRef<Http>>>,
    channel: u64,
    content: Option<Map<String, Value>>,
    content_overflow: u16,
    // embed_overflow: u16,
    reactions: Option<Vec<ReactionType>>,
    attachments: Option<Vec<AttachmentType<'a>>>,
    fut: Option<BoxedFuture<'a, Result<Message>>>,
}

impl<'a> SendMessageBuilder<'a> {
    pub fn new(http: impl AsRef<Http>, channel: u64) -> Self {
        Self {
            http: Some(Box::new(http)),
            channel,
            content: Some(Default::default()),
            fut: None,
            attachments: None,
            reactions: None,
            content_overflow: 0,
        }
    }
    
    pub fn content(self, content: impl ToString) -> Self {
        let content = content.to_string();
        let len = u16::try_from(content.count()).unwrap_or(u16::MAX);
        
        if len > 2000 {
            self.content_overflow = len - 2000;
        } else {
            self.content_overflow = 0;
            self.content.as_mut().insert(String::from("content"), content.to_string().into());
        }
        
        self
    }
    
    pub fn add_file(self, file: impl Into<AttachmentType<'a>>) -> Self {
        self.attachments.extend_inner(file.into());
        self
    }
    
    pub fn add_files<F: Into<AttachmentType<'a>>, I: IntoIterator<Item=F>>(self, files: I) -> Self {
        for attachment in files {
            self.attachments.extend_inner(attachment.into());
        }
        
        self
    }
    
    pub fn embed<F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed>(self, f: F) -> Self {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        
        let embed = serenity::utils::hashmap_to_json_map(embed.0);
        self.content.as_mut().insert(String::from("embed"), embed.into());
            
        self
    }
    
    pub fn reactions<R: Into<ReactionType>, I: IntoIterator<Item=R>>(self, reactions: I) -> Self {
        for reaction in reactions {
            self.reactions.extend_inner(reaction.into());
        }
        
        self
    }
    
    pub fn tts(self, tts: bool) -> Self {
        self.content.as_mut().insert(String::from("tts"), tts.into());
        self
    }
}

impl<'a> Future for SendMessageBuilder<'a> {
    type Output = Result<Message>;
    
    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.fut {
            Some(ref mut f) => f.as_mut().poll(ctx),
            None => {
                let http = self.http.take().unwrap().as_ref<Http>();
                let content = self.content.take().unwrap();
                
                if self.attachments.is_some() {
                    if let Some(e) = content.remove("embed") {
                        let name = String::from("payload_json");
                        if let Some(c) = content.remove("content") {
                            content.insert(name, json!({ "content": c, "embed": e }));
                        } else {
                            content.insert(name, json!({ "embed": e }));
                        }
                    }
                }
                
                let attachments = self.attachments.take();
            
                self.fut = Some(Box::pin(async move {
                    let content = content.into();
                    let message = match attachments {
                        Some(a) => http.send_files(self.channel, a, self.content).await?,
                        None => http.send_message(self.channel, &self.content.into()).await?
                    };
                    
                    if let Some(reactions) = &self.reactions {
                        for reaction in reactions {
                            http.create_reaction(self.channel, message.id.0, &reaction).await?;
                        }
                    }
                    
                    Ok(message)
                }));
                
                self.fut.as_mut().unwrap().as_mut().poll(ctx)
            }
        }
    }
}

pub trait ChannelExt: Into<ChannelId> {
    #[inline]
    fn send_message(&self, http: impl AsRef<Http>) -> SendMessageBuilder {
        SendMessageBuilder::new(http, self.into().0)
    }
}

impl<C: Into<ChannelId>> ChannelExt for C {}