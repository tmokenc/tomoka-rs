use async_trait::async_trait;

use serenity::{
    model::{channel::Message, event::Event},
    prelude::*,
};

use std::sync::atomic::{AtomicU64, AtomicBool};

use tokio::sync::RwLock;

#[derive(Default, Debug)]
pub struct TuiEventHandler {
    messages: RwLock<Vec<Message>>,
    ready: AtomicBool,
    locked: AtomicBool,
    locked_channel: AtomicU64,
}

impl TuiEventHandler {
    pub fn new() -> Self {
        Default::default()
    }
}

#[async_trait]
impl RawEventHandler for TuiEventHandler {
    async fn raw_event(&self, ctx: Context, ev: Event) {
        match ev {
            Event::MessageCreate(e) => {
                self.messages.write().await.push(e.message);
            }

            Event::MessageUpdate(_e) => {
                todo!()
            }

            Event::MessageDelete(_e) => {
                todo!()
            }

            Event::MessageDeleteBulk(_e) => {
                todo!()
            }

            Event::Ready(_e) => {
                todo!()
            }

            _ => {}
        }
    }
}
