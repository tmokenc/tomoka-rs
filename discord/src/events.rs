use async_trait::async_trait;
use serenity::client::{Context, EventHandler, RawEventHandler};
use serenity::model::{
    channel::{Channel, Message},
    event::{Event, MessageUpdateEvent, ResumedEvent},
    gateway::{Activity, Ready},
    id::{ChannelId, GuildId, MessageId},
    user::OnlineStatus,
};

use crate::cache::MessageCache;
use crate::storages::CacheStorage;
use crate::{utils::*, Result};

use colorful::RGB;
use colorful::{Color, Colorful};
use futures::future::BoxFuture;
use futures::future::{self, FutureExt, TryFutureExt};
use magic::number_to_rgb;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time;

static CURRENT_CHANNEL: AtomicU64 = AtomicU64::new(450521152272728065);
static LOCKED: AtomicBool = AtomicBool::new(false);

type EventHook = for<'fut> fn(&'fut Context, &'fut Event) -> BoxFuture<'fut, Result<()>>;
pub struct RawHandler {
    pub handler: Arc<RawEvents>,
}

impl RawHandler {
    pub fn new() -> Self {
        Self {
            handler: Arc::new(RawEvents::new()),
        }
    }
}

#[async_trait]
impl RawEventHandler for RawHandler {
    async fn raw_event(&self, ctx: Context, ev: Event) {
        if let Event::Unknown(event) = &ev {
            debug!("An unknown event\n{:#?}", event);
        }

        self.handler.execute(ctx, ev).await;
    }
}

pub struct RawEvents {
    events: RwLock<HashMap<String, EventHook>>,
    actions: Mutex<Vec<Action>>,
}

enum Action {
    Add(String, EventHook),
    Remove(String),
}

impl RawEvents {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(HashMap::new()),
            actions: Mutex::new(Vec::new()),
        }
    }

    pub async fn add(&self, name: impl ToString, fut: EventHook) {
        let timeout = Duration::from_millis(50);
        let name = name.to_string();

        match time::timeout(timeout, self.events.write()).await {
            Ok(ref mut events) => {
                events.insert(name, fut);
            }
            Err(_) => self.actions.lock().await.push(Action::Add(name, fut)),
        }
    }

    pub async fn remove(&self, name: impl ToString) {
        let timeout = Duration::from_millis(50);
        let name = name.to_string();

        match time::timeout(timeout, self.events.write()).await {
            Ok(ref mut events) => {
                events.remove(&name);
            }
            Err(_) => self.actions.lock().await.push(Action::Remove(name)),
        }
    }

    pub async fn execute(&self, ctx: Context, ev: Event) {
        let handlers = self.events.read().await;
        let fut = handlers
            .iter()
            .map(|(k, v)| v(&ctx, &ev).map_err(move |e| (k, e)).boxed());

        future::join_all(fut)
            .await
            .into_iter()
            .filter_map(|f| f.err())
            .for_each(|(n, e)| error!("Cannot execute the event {}\n{:#?}", n, e));

        drop(handlers);

        let timeout = Duration::from_millis(1);
        if let Ok(ref mut map) = time::timeout(timeout, self.events.write()).await {
            for action in self.actions.lock().await.drain(..) {
                match action {
                    Action::Add(name, fut) => map.insert(name, fut),
                    Action::Remove(name) => map.remove(&name),
                };
            }
        }
    }
}

pub struct Handler {
    ready: AtomicU64,
    resume: AtomicU64,
    connected: AtomicBool,
}
impl Handler {
    pub fn new() -> Self {
        Self {
            ready: AtomicU64::from(0),
            resume: AtomicU64::from(0),
            connected: AtomicBool::from(false),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.author.bot {
            let mut to_log = true;

            if LOCKED.load(Ordering::SeqCst) {
                to_log = CURRENT_CHANNEL.load(Ordering::SeqCst) == msg.channel_id.0;
            } else {
                CURRENT_CHANNEL.store(msg.channel_id.0, Ordering::SeqCst);
            }

            if to_log {
                let channel_info: String = get_colored_channel_info(&ctx, msg.channel_id).await;
                trace!(
                    "A message on {}\n{}> {}",
                    channel_info,
                    colored_name_user(&msg.author).underlined(),
                    msg.content.to_owned().gradient(Color::LightGreen),
                );
            }
        }

        let will_be_cached = is_watching_channel(&ctx, msg.channel_id).await
            && (!msg.content.is_empty() || !msg.attachments.is_empty())
            && !msg.is_own(&ctx).await;

        if will_be_cached {
            let cache = get_data::<CacheStorage>(&ctx).await.unwrap();
            cache.insert_message(msg).await;
        }
    }

    async fn message_update(
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
        let guild_id = match ctx.cache.read().await.guild_channel(channel_id.0) {
            Some(channel) => channel.read().await.guild_id,
            None => return,
        };

        trace!(
            "The message with id {} on channel {} has been updated",
            event.id.0,
            channel_id.0
        );

        let log_channel = match get_log_channel(guild_id).await {
            Some(channel) => channel,
            None => return,
        };

        let mut fields = vec![("Updated message", content.clone(), false)];
        let mut to_say = format!(
            "A [message]({}) by **{}**#{:04} on channel <#{}> has been edited.",
            format_args!(
                "https://discordapp.com/channels/{}/{}/{}",
                guild_id, channel_id, event.id
            ),
            author.name,
            author.discriminator,
            channel_id.0,
        );

        let cache = get_data::<CacheStorage>(&ctx).await.unwrap();
        match cache.update_message(event.id, &content).await {
            Some(old_message) => {
                fields.insert(0, ("Original message", old_message, false));
            }
            None => to_say.push_str("\nBut I cannot remember how it was..."),
        };

        let color = crate::read_config().await.color.message_update;

        let send = log_channel.send_message(&ctx, |m| {
            m.embed(|embed| {
                embed.description(to_say);
                embed.timestamp(now());
                embed.fields(fields);
                embed.color(color);

                embed
            })
        });

        if let Err(why) = send.await {
            error!("Cannot send the message update log\n{:#?}", why);
        }
    }

    async fn message_delete(&self, ctx: Context, channel: ChannelId, msg: MessageId) {
        trace!("A message with id {} has been deleted", msg.0);
        process_deleted_message(&ctx, channel, Some(msg)).await; // Option also an iterator
    }

    async fn message_delete_bulk(&self, ctx: Context, channel_id: ChannelId, msgs: Vec<MessageId>) {
        trace!(
            "OMG, there are {} messages has been killed by one slash",
            msgs.len()
        );

        process_deleted_message(&ctx, channel_id, msgs.into_iter().rev()).await
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!(
            "{} is now available on {} servers",
            ready.user.name,
            ready.guilds.len()
        );

        let mess = {
            let resume = self.resume.load(Ordering::SeqCst);
            let count = self.ready.fetch_add(1, Ordering::SeqCst) + 1;
            format!("tmokenc#0001 ({}/{})", resume, count)
        };

        let activity = Activity::listening(&mess);
        let status = OnlineStatus::DoNotDisturb;

        ctx.set_presence(Some(activity), status).await;

        if !self.connected.load(Ordering::Relaxed) {
            self.connected.store(true, Ordering::SeqCst);

            {
                let (config, shard) = future::join(crate::read_config(), ctx.shard.lock()).await;

                let ids = config.guilds.iter().map(|v| *v.key());
                shard.chunk_guilds(ids, None, None);
            }

            if let Ok(info) = ctx.http.get_current_application_info().await {
                crate::write_config().await.masters.insert(info.owner.id);
            }

            let arc_ctx = Arc::new(ctx);
            tokio::spawn(read_input(arc_ctx));
        }
    }

    async fn resume(&self, ctx: Context, resume: ResumedEvent) {
        let mess = {
            let count = self.resume.fetch_add(1, Ordering::SeqCst) + 1;
            let ready = self.ready.load(Ordering::SeqCst);
            format!("tmokenc#0001 ({}/{})", count, ready)
        };

        let activity = Activity::listening(&mess);
        let status = OnlineStatus::DoNotDisturb;

        ctx.set_presence(Some(activity), status).await;
        debug!("Resumed; trace: {:?}", resume.trace);
    }
}

async fn is_watching_channel(ctx: &Context, channel: ChannelId) -> bool {
    let guild_id = get_guild_id_from_channel(ctx, channel).await;
    match guild_id {
        Some(v) => get_log_channel(v).await.is_some(),
        None => false,
    }
}

async fn get_log_channel(guild: GuildId) -> Option<ChannelId> {
    crate::read_config()
        .await
        .guilds
        .get(&guild)
        .and_then(|config| config.logger.channel.filter(|_| config.logger.enable))
        .map(ChannelId)
}

async fn get_colored_channel_info(ctx: &Context, c: ChannelId) -> String {
    use Channel::*;

    match c.to_channel(ctx).await {
        Ok(Guild(channel)) => {
            let guild_name = channel
                .guild(&ctx.cache)
                .await
                .unwrap()
                .read()
                .await
                .name
                .to_owned();

            format!(
                "channel {}({}) at server {}",
                channel.name.to_owned().color(to_color(channel.id.0)),
                channel.id.0,
                guild_name.color(to_color(channel.guild_id.0))
            )
        }

        Ok(Private(_c)) => format!("private message ({})", c.0),
        Ok(Category(_c)) => format!("a category...? ({})", c.0),
        _ => String::from("Unknown"),
    }
}

async fn process_deleted_message<I>(ctx: &Context, channel_id: ChannelId, msgs: I)
where
    I: IntoIterator<Item = MessageId>,
{
    let guild_id = match get_guild_id_from_channel(&ctx, channel_id).await {
        Some(id) => id,
        None => return,
    };

    let log_channel = match get_log_channel(guild_id).await {
        Some(c) => c,
        None => return,
    };

    let cache = get_data::<CacheStorage>(&ctx).await.unwrap();

    for msg in msgs {
        let mess = match cache.remove_message(msg).await {
            Some(v) => v,
            None => continue,
        };

        if let Err(why) = _process_deleted(&ctx, log_channel, channel_id, mess).await {
            error!("Cannot log deleted message\n{:#?}", why);
        }
    }
}

#[rustfmt::skip]
async fn _process_deleted(
    ctx: &Context,
    log_channel: ChannelId,
    channel_id: ChannelId,
    msg: MessageCache,
) -> Result<()> {
    let is_empty_content = msg.content.is_empty();
    if is_empty_content && msg.attachments.is_empty() {
        return Ok(());
    }
    
    let (name, discriminator, is_bot) = match ctx.cache.read().await.user(msg.author_id) {
        None => ("Unknown".to_string(), 0, false),
        Some(user) => {
            let info = user.read().await;
            (info.name.to_owned(), info.discriminator, info.bot)
        }
    };
    
    let color = crate::read_config().await.color.message_delete;
    
    let mut fields = Vec::new();
       
    let typed = if is_empty_content { "file" } else {
        trace!("{}", msg.content);
        fields.push(("Deleted message", msg.content.to_owned(), false));
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

    log_channel.send_message(&ctx, |message| {
        message.embed(|embed| {
            embed.description(content);
            embed.timestamp(now());
            embed.fields(fields);
            embed.color(color);
            
            embed
        });
        
        message
    }).await?;
    
    if !msg.attachments.is_empty() {
        log_channel.send_message(ctx, |message| {
            msg.attachments
                .iter()
                .filter_map(|v| v.cached.as_ref())
                .for_each(|v| { message.add_file(v); });
            
            message
        }).await?;
    }

    Ok(())
}

fn to_color(id: u64) -> RGB {
    let (r, g, b) = number_to_rgb(id);
    RGB::new(r, g, b)
}

async fn read_input(ctx: Arc<Context>) {
    use tokio::io::AsyncBufReadExt;

    let stdin = tokio::io::stdin();
    let reader = tokio::io::BufReader::new(stdin);
    let mut lines = reader.lines();

    let mut data = InputData {
        messages: Vec::new(),
        max_history: 10,
    };

    loop {
        if let Ok(Some(line)) = lines.next_line().await {
            if let Ok(i) = Input::parse(&line) {
                if let Err(why) = process_input(Arc::clone(&ctx), &i, &mut data).await {
                    error!("{:?} > {:?}", i, why)
                }
            }
        }
    }
}

async fn process_input<'a>(
    ctx: Arc<Context>,
    input: &Input<'a>,
    data: &mut InputData,
) -> Result<()> {
    match input {
        Input::Message(s) => {
            let channel = ChannelId(CURRENT_CHANNEL.load(Ordering::SeqCst));
            channel.broadcast_typing(&ctx).await?;
            
            time::delay_for(Duration::from_millis(1500)).await;
            let msg = channel.say(ctx, s).await?;
            data.messages.push((channel, msg.id));
            
            if data.messages.len() > data.max_history {
                data.messages.remove(0);
            }
        }

        Input::Edit(s) => {
            if let Some((channel, msg)) = data.messages.last() {
                channel.edit_message(ctx, msg, |m| m.content(s)).await?;
            }
        }

        Input::Delete(v) => {
            let (channel, msg) = match v {
                Some(v) => *v,
                None => data.messages.pop().ok_or(magic::MagicError)?,
            };

            channel.delete_message(ctx, msg).await?;
        }

        Input::Lock(c) => {
            LOCKED.store(true, Ordering::SeqCst);
            if let Some(channel) = c {
                CURRENT_CHANNEL.store(channel.0, Ordering::SeqCst);
            }
        }

        Input::Unlock => LOCKED.store(false, Ordering::SeqCst),
    }

    Ok(())
}

#[derive(Debug)]
pub struct InputData {
    messages: Vec<(ChannelId, MessageId)>,
    max_history: usize,
}

#[derive(Debug)]
pub enum Input<'a> {
    Message(&'a str),
    Edit(&'a str),
    Delete(Option<(ChannelId, MessageId)>),
    Lock(Option<ChannelId>),
    Unlock,
}

impl<'a> Input<'a> {
    pub fn parse(s: &'a str) -> std::result::Result<Self, magic::Void> {
        let mut split = s.split_whitespace();

        match split.next() {
            Some(":delete") => {
                // todo: delete ids
                Ok(Self::Delete(None))
            }

            Some(":lock") => {
                let channel = split
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(ChannelId);

                Ok(Self::Lock(channel))
            }

            Some(":edit") => split.next().map(|_| Self::Edit(&s[6..])).ok_or(magic::Void),
            Some(":unlock") => Ok(Self::Unlock),
            Some(_) => Ok(Self::Message(s)),
            None => Err(magic::Void),
        }
    }
}
