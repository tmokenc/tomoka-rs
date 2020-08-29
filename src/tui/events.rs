use async_trait::async_trait;

use serenity::{
    model::{channel::Message, event::Event},
    prelude::*,
};

use tokio::sync::RwLock;

pub struct TuiEventHandler {
    messages: RwLock<Vec<Message>>,
}

#[async_trait]
impl RawEventHandler for TuiEventHandler {
    async fn raw_event(&self, ctx: Context, ev: Event) {
        match ev {
            Event::MessageCreate(e) => {
                self.messages.write().await.push(e.message);
            }
        }
    }
}
