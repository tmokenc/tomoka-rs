use serenity::client::{Context, EventHandler, RawEventHandler};
use serenity::model::{
    channel::{Channel, Message},
    event::{Event, MessageUpdateEvent, ResumedEvent},
    gateway::{Activity, Ready},
    id::{ChannelId, GuildId, MessageId},
    user::OnlineStatus,
};

use crate::cache::MessageCache;
use crate::storages::{CacheStorage};
use crate::{types::CustomEvents, utils::*, Result};

use chrono::Utc;
use colorful::RGB;
use colorful::{Color, Colorful};
use magic::number_to_rgb;
use std::sync::Arc;

pub struct RawHandler {
    pub custom_events: Arc<CustomEvents>,
}

impl RawHandler {
    pub fn new() -> Self {
        Self {
            custom_events: Arc::new(CustomEvents::new()),
        }
    }
}

impl RawEventHandler for RawHandler {
    fn raw_event(&self, ctx: Context, ev: Event) {
        if let Event::Unknown(e) = &ev {
            dbg!(e);
        }

        self.custom_events.execute(&ctx, &ev);
    }
}

pub struct Handler;
impl Handler {
    pub fn new() -> Self {
        Self
    }
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        if !msg.author.bot {
            let channel_info: String = get_colored_channel_info(&ctx, msg.channel_id);

            info!(
                "A message on {}\n{}> {}",
                channel_info,
                colored_name_user(&msg.author).underlined(),
                msg.content.to_owned().gradient(Color::LightGreen),
            );
        }

        let will_be_cached = is_watching_channel(&ctx, msg.channel_id)
            && (!msg.content.is_empty() || !msg.attachments.is_empty())
            && !msg.is_own(&ctx);

        if will_be_cached {
            let cache = get_data::<CacheStorage>(&ctx).unwrap();
            cache.insert_message(msg);
        }
    }

    fn message_update(
        &self,
        ctx: Context,
        _old: Option<Message>,
        _new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        let content = match event.content.filter(|v| !v.is_empty()) {
            Some(c) => c,
            None => return,
        };

        let author = match event.author {
            Some(a) => a,
            None => return,
        };

        let channel_id = event.channel_id;
        let guild_id = match ctx.cache.read().guild_channel(channel_id.0) {
            Some(channel) => channel.read().guild_id,
            None => return,
        };

        info!(
            "The message with id {} on channel {} has been updated",
            event.id.0, channel_id.0
        );

        let log_channel = match get_log_channel(guild_id) {
            Some(channel) => channel,
            None => return,
        };

        let mut fields = vec![("Updated message", content.clone(), false)];
        let mut to_say = format!(
            "A message by **{}**#{:04} on channel <#{}> has been edited.",
            author.name, author.discriminator, channel_id.0,
        );

        let cache = get_data::<CacheStorage>(&ctx).unwrap();
        match cache.update_message(event.id, &content) {
            Some(old_message) => {
                fields.insert(0, ("Original message", old_message, false));
            }
            None => to_say.push_str("\nBut I cannot remember how it was..."),
        };

        send(&ctx.http, log_channel, to_say, |embed| {
            embed.timestamp(now());
            embed.fields(fields);

            {
                let config = crate::read_config();
                embed.color(config.color.message_update);
            }

            embed
        });
    }

    fn message_delete(&self, ctx: Context, channel: ChannelId, msg: MessageId) {
        info!("A message with id {} has been deleted", msg.0);
        process_deleted_message(&ctx, channel, Some(msg)); // Option also an iterator
    }

    fn message_delete_bulk(&self, ctx: Context, channel_id: ChannelId, msgs: Vec<MessageId>) {
        info!(
            "OMG, there are {} messages has been killed by one slash",
            msgs.len()
        );

        process_deleted_message(&ctx, channel_id, msgs.into_iter().rev())
    }

    fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        info!(
            "The bot is now available on {} servers and {} private channels",
            ready.guilds.len(),
            ready.private_channels.len(),
        );

        let activity = Activity::listening(&crate::read_config().prefix);
        let status = OnlineStatus::DoNotDisturb;

        ctx.set_presence(Some(activity), status);

        if let Ok(info) = ctx.http.get_current_application_info() {
            crate::write_config().masters.insert(info.owner.id);
        }
    }

    fn resume(&self, _: Context, resume: ResumedEvent) {
        debug!("Resumed; trace: {:?}", resume.trace);
    }
}

fn is_watching_channel(ctx: &Context, channel: ChannelId) -> bool {
    get_guild_id_from_channel(ctx, channel)
        .and_then(get_log_channel)
        .is_some()
}

fn get_log_channel(guild: GuildId) -> Option<ChannelId> {
    crate::read_config()
        .guilds
        .get(&guild)
        .and_then(|config| config.logger.channel.filter(|_| config.logger.enable))
}

fn get_colored_channel_info(ctx: &Context, c: ChannelId) -> String {
    use Channel::*;

    match c.to_channel(ctx) {
        Ok(Guild(c)) => {
            let channel = c.read();
            let guild_name = channel.guild(&ctx.cache).unwrap().read().name.to_owned();

            format!(
                "channel {}({}) at server {}",
                channel.name.to_owned().color(to_color(channel.id.0)),
                channel.id.0,
                guild_name.color(to_color(channel.guild_id.0))
            )
        }
        Ok(Group(c)) => {
            let channel = c.read();
            let color = to_color(channel.channel_id.0);

            let owner = channel
                .owner_id
                .to_user(ctx)
                .ok()
                .map_or("Unknown".bold().underlined(), |user| {
                    colored_name_user(&user)
                });

            if let Some(name) = channel.name.to_owned() {
                format!(
                    "the group {}({}) started by {}",
                    name.color(color),
                    channel.channel_id.0,
                    owner
                )
            } else {
                format!(
                    "a {}({}) started by {}",
                    "group".color(color),
                    channel.channel_id.0,
                    owner
                )
            }
        }
        Ok(Private(_c)) => format!("private message ({})", c.0),
        Ok(Category(_c)) => format!("a category...? ({})", c.0),
        _ => String::from("Unknown"),
    }
}

fn process_deleted_message<I>(ctx: &Context, channel_id: ChannelId, msgs: I)
where
    I: IntoIterator<Item = MessageId>,
{
    let guild_id = match get_guild_id_from_channel(&ctx, channel_id) {
        Some(id) => id,
        None => return,
    };

    let log_channel = match get_log_channel(guild_id) {
        Some(c) => c,
        None => return,
    };

    let cache = get_data::<CacheStorage>(&ctx).unwrap();

    msgs.into_iter()
        .filter_map(|v| cache.remove_message(v))
        .filter_map(|v| _process_deleted(&ctx, log_channel, channel_id, v).err())
        .for_each(|err| error!("Cannot log deleted message\n{:#?}", err))
}

#[rustfmt::skip]
fn _process_deleted(
    ctx: &Context,
    log_channel: ChannelId,
    channel_id: ChannelId,
    msg: MessageCache,
) -> Result<()> {
    let is_empty_content = msg.content.is_empty();
    
    if is_empty_content && msg.attachments.is_empty() {
        return Ok(());
    }
    
    info!("{}", msg.content.to_owned().dim());
    
    let (name, discriminator, is_bot) = match ctx.cache.read().user(msg.author_id) {
        None => ("Unknown".to_string(), 0, false),
        Some(user) => {
            let info = user.read();
            (info.name.to_owned(), info.discriminator, info.bot)
        }
    };

    log_channel.send_message(&ctx, |message| {
        msg.attachments
            .iter()
            .filter_map(|v| v.cached.as_ref())
            .for_each(|v| { message.add_file(v); });
            
        let color = {
            let config = crate::read_config();
            config.color.message_delete
        };

        let typed = if is_empty_content { "file" } else {
            message.embed(|embed| {
                embed
                    .color(color)
                    .timestamp(Utc::now().to_rfc3339())
                    .field("Deleted message", msg.content.to_owned(), false)
            });
    
            "message"
        };

        let content = format!(
            "A {} by {} **{}**#{:04} on channel <#{}> has been deleted",
            typed,
            if is_bot { "a *bot* named" } else { "" },
            name,
            discriminator,
            channel_id.0
        );

        message.content(content)
    })?;

    Ok(())
}

fn to_color(id: u64) -> RGB {
    let (r, g, b) = number_to_rgb(id);
    RGB::new(r, g, b)
}
