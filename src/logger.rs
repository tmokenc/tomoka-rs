use crate::traits::RawEventHandlerRef;
use serenity::client::Context;
use serenity::model::event::Event;

pub struct EventLogger;

impl EventLogger {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl RawEventHandlerRef for EventLogger {
    async fn raw_event_ref(&self, _ctx: &Context, ev: &Event) {
        match ev {
            Event::Ready(e) => {
                log::info!(
                    "{} is now available on {} servers",
                    e.ready.user.name,
                    e.ready.guilds.len()
                );
            }

            Event::Resumed(e) => {
                log::debug!("Resumed; trace: {:?}", e.trace);
            }

            Event::Unknown(e) => log::debug!("An unknown event from discord\n{:#?}", e),
            _ => {}
        }
    }
}
