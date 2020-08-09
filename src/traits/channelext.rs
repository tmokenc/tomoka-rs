use serenity::Result;
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
use core::mem;
use std::sync::Arc;
use std::pin::Pin;
use std::task::{Poll, Context};

type BoxedFuture<'a, T> = Pin<Box<dyn core::future::Future<Output = T> + 'a + Send>>;

pub trait ChannelExt: Into<ChannelId> + Clone {
    #[inline]
    fn send_message<'a>(&self, http: impl AsRef<Arc<Http>>) -> SendMessageBuilder<'a> {
        SendMessageBuilder::new(Arc::clone(http.as_ref()), self.to_owned().into())
    }
    
    #[inline]
    fn send_embed<'a>(&self, http: impl AsRef<Arc<Http>>) -> SendEmbedBuilder<'a> {
        SendEmbedBuilder::new(Arc::clone(http.as_ref()), self.to_owned().into())
    }
}

impl<C: Into<ChannelId> + Clone> ChannelExt for C {}

pub struct SendMessageBuilder<'a> {
    http: Option<Arc<Http>>,
    channel: u64,
    content: Option<Map<String, Value>>,
    content_overflow: u16,
    // embed_overflow: u16,
    reactions: Option<Vec<ReactionType>>,
    attachments: Option<Vec<AttachmentType<'a>>>,
    fut: Option<BoxedFuture<'a, Result<Message>>>,
}

impl<'a> SendMessageBuilder<'a> {
    pub fn new(http: Arc<Http>, channel: ChannelId) -> Self {
        Self {
            http: Some(http),
            channel: channel.0,
            content: Some(Default::default()),
            fut: None,
            attachments: None,
            reactions: None,
            content_overflow: 0,
        }
    }
    
    pub fn with_content(mut self, content: impl ToString) -> Self {
        let content = content.to_string();
        let len = u16::try_from(content.count()).unwrap_or(u16::MAX);
        
        if len > 2000 {
            self.content_overflow = len - 2000;
        } else {
            self.content_overflow = 0;
            self.content.get_or_insert_with(Map::new).insert(String::from("content"), content.to_string().into());
        }
        
        self
    }
    
    pub fn with_file(mut self, file: impl Into<AttachmentType<'a>>) -> Self {
        self.attachments.extend_inner(file.into());
        self
    }
    
    pub fn with_files<F: Into<AttachmentType<'a>>, I: IntoIterator<Item=F>>(mut self, files: I) -> Self {
        for attachment in files {
            self.attachments.extend_inner(attachment.into());
        }
        
        self
    }
    
    pub fn with_embed<F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed>(mut self, f: F) -> Self {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        
        let embed = serenity::utils::hashmap_to_json_map(embed.0);
        self.content
            .get_or_insert_with(Map::new)
            .insert(String::from("embed"), embed.into());
            
        self
    }
    
    pub fn with_reactions<R: Into<ReactionType>, I: IntoIterator<Item=R>>(mut self, reactions: I) -> Self {
        for reaction in reactions {
            self.reactions.extend_inner(reaction.into());
        }
        
        self
    }
    
    pub fn is_tts(mut self, tts: bool) -> Self {
        self.content.get_or_insert_with(Map::new).insert(String::from("tts"), tts.into());
        self
    }
}

impl<'a> Future for SendMessageBuilder<'a> {
    type Output = Result<Message>;
    
    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.fut {
            Some(ref mut f) => f.as_mut().poll(ctx),
            None => {
                let http = self.http.take().unwrap();
                let mut content = self.content.take().unwrap();
                
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
                let channel = self.channel;
                let reactions = self.reactions.to_owned();
            
                self.fut = Some(Box::pin(async move {
                    let message = match attachments {
                        Some(a) => http.send_files(channel, a, content).await?,
                        None => http.send_message(channel, &content.into()).await?
                    };
                    
                    if let Some(reactions) = &reactions {
                        for reaction in reactions {
                            http.create_reaction(channel, message.id.0, &reaction).await?;
                        }
                    }
                    
                    Ok(message)
                }));
                
                self.fut.as_mut().unwrap().as_mut().poll(ctx)
            }
        }
    }
}

pub struct SendEmbedBuilder<'a> {
    http: Option<Arc<Http>>,
    channel: u64,
    embed: CreateEmbed,
    fut: Option<BoxedFuture<'a, Result<Message>>>,
}

macro_rules! basic_impl {
    ($n:ident, $t:ident) => {
        pub fn $n<S: ToString>(mut self, $t: S) -> Self {
            self.embed.$t($t);
            self
        }
    }
}

impl SendEmbedBuilder<'_> {
    #[inline]
    pub fn new(http: Arc<Http>, channel: ChannelId) -> Self {
        Self {
            http: Some(http),
            channel: channel.0,
            embed: CreateEmbed::default(),
            fut: None,
        }
    }
    
    basic_impl!(with_title, title);
    basic_impl!(with_url, url);
    basic_impl!(with_attachment, attachment);
    basic_impl!(with_description, description);
    basic_impl!(with_image, image);
    basic_impl!(with_thumbnail, thumbnail);
    
    pub fn with_field<N: ToString, S: ToString>(mut self, name: N, value: S, inline: bool) -> Self {
        self.embed.field(name, value, inline);
        self
    }
    
    pub fn with_fields<N: ToString, S: ToString, I: IntoIterator<Item=(N, S, bool)>>(mut self, iter: I) -> Self {
        self.embed.fields(iter);
        self
    }
    
    pub fn with_color(mut self, color: u32) -> Self {
        self.embed.color(color);
        self
    }
    
    pub fn with_timestamp(mut self, timestamp: u32) -> Self {
        todo!();
    }
    
    #[inline]
    pub fn inner_embed(&mut self) -> &mut CreateEmbed {
        &mut self.embed
    }
}

impl<'a> Future for SendEmbedBuilder<'a> {
    type Output = Result<Message>;
    
    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.fut.as_mut() {
            Some(ref mut future) => future.as_mut().poll(ctx),
            None => {
                let http = self.http.take().unwrap();
                let channel = self.channel;
                let embed = mem::take(&mut self.embed);
                let embed = serenity::utils::hashmap_to_json_map(embed.0);
                
                let mut content: Map<String, Value> = Map::new();
                content.insert(String::from("embed"), embed.into());
                
                self.fut = Some(Box::pin(async move {
                    http.send_message(channel, &content.into()).await
                }));
                
                self.fut.as_mut().unwrap().as_mut().poll(ctx)
            }
        }
    }
}
