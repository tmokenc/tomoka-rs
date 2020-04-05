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
use magic::number_to_rgb;
use std::sync::Arc;

pub struct Handler;
impl Handler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.author.bot {
            let channel_info: String = get_colored_channel_info(&ctx, msg.channel_id).await;

            info!(
                "A message on {}\n{}> {}",
                channel_info,
                colored_name_user(&msg.author).underlined(),
                msg.content.to_owned().gradient(Color::LightGreen),
            );
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

        info!(
            "The message with id {} on channel {} has been updated",
            event.id.0, channel_id.0
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

        let color = {
            let config = crate::read_config().await;
            config.color.message_update
        };

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
        info!("A message with id {} has been deleted", msg.0);
        process_deleted_message(&ctx, channel, Some(msg)).await; // Option also an iterator
    }

    async fn message_delete_bulk(&self, ctx: Context, channel_id: ChannelId, msgs: Vec<MessageId>) {
        info!(
            "OMG, there are {} messages has been killed by one slash",
            msgs.len()
        );

        process_deleted_message(&ctx, channel_id, msgs.into_iter().rev()).await
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        info!(
            "The bot is now available on {} servers and {} private channels",
            ready.guilds.len(),
            ready.private_channels.len(),
        );

        // let activity = Activity::listening(&crate::read_config().prefix);
        let activity = Activity::listening("tomo>leaderboard");
        let status = OnlineStatus::DoNotDisturb;

        ctx.set_presence(Some(activity), status).await;

        if let Ok(info) = ctx.http.get_current_application_info().await {
            crate::write_config().await.masters.insert(info.owner.id);
        }
    }

    async fn resume(&self, _: Context, resume: ResumedEvent) {
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
}

async fn get_colored_channel_info(ctx: &Context, c: ChannelId) -> String {
    use Channel::*;

    match c.to_channel(ctx).await {
        Ok(Guild(c)) => {
            let channel = c.read().await;
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
            None => return,
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
    
    info!("{}", msg.content.to_owned().dim());
    
    let (name, discriminator, is_bot) = match ctx.cache.read().await.user(msg.author_id) {
        None => ("Unknown".to_string(), 0, false),
        Some(user) => {
            let info = user.read().await;
            (info.name.to_owned(), info.discriminator, info.bot)
        }
    };
    
    let color = {
        let config = crate::read_config().await;
        config.color.message_delete
    };

    log_channel.send_message(&ctx, |message| {
        let mut fields = Vec::new();
           
        let typed = if is_empty_content { "file" } else {
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
        
        message.embed(|embed| {
            embed.description(content);
            embed.timestamp(now());
            embed.fields(fields);
            embed.color(color);
            
            embed
        });
        
        message
    }).await?;
    
    log_channel.send_message(ctx, |message| {
        msg.attachments
            .iter()
            .filter_map(|v| v.cached.as_ref())
            .for_each(|v| { message.add_file(v); });
        
        message
    }).await?;

    Ok(())
}

fn to_color(id: u64) -> RGB {
    let (r, g, b) = number_to_rgb(id);
    RGB::new(r, g, b)
}
