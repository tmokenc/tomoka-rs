#![allow(unstable_name_collisions)]

use serenity::builder::CreateEmbed;
use serenity::client::Context;
use serenity::framework::standard::macros::hook;
use serenity::framework::{
    standard::{
        help_commands, macros::help, Args, CommandGroup, CommandResult, Configuration, HelpOptions,
    },
    Framework, StandardFramework,
};
use serenity::model::{
    channel::{Message, ReactionType},
    id::{MessageId, UserId},
    misc::EmojiIdentifier,
};

use crate::{
    commands::*,
    storages::{AIStore, InforKey, ReqwestClient},
    traits::ToEmbed,
    utils::*,
    Result,
};

use chrono::{DateTime, Utc};
use colorful::Colorful;
use core::time::Duration;
use dashmap::DashMap;
use lazy_static::lazy_static;
use log::{error, info};
use magic::has_external_command;
use requester::ehentai::EhentaiApi;
use smallstr::SmallString;
use std::collections::HashSet;

use magic::traits::MagicBool as _;
use magic::traits::MagicIter as _;
use magic::traits::MagicStr as _;
use std::fmt::Write as _;

const TYPING_LIST: &[&str] = &["diancie", "say", "flip", "rotate", "saucenao", "info"];

lazy_static! {
    static ref EXECUTION_LIST: DashMap<MessageId, DateTime<Utc>> = DashMap::new();
}

#[help]
#[individual_command_tip = "Hello! こんにちは！\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[embed_success_colour(MEIBE_PINK)]
#[embed_error_colour(ROSEWATER)]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn stolen_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await
}

pub fn get_framework() -> impl Framework {
    let mut framework = StandardFramework::new()
        // .bucket("basic", |b| b.delay(2).time_span(10).limit(3))
        // .await
        .group(&MASTER_GROUP)
        .group(&GUILDMASTER_GROUP)
        .group(&ADMINISTRATION_GROUP)
        .group(&CORONA_GROUP)
        .group(&GENERAL_GROUP)
        // .group(&GAME_GROUP)
        .group(&POKEMON_GROUP)
        .group(&UTILITY_GROUP)
        .group(&IMAGE_GROUP)
        .group(&RGB_GROUP)
        .help(&STOLEN_HELP)
        .configure(framwork_config)
        .before(before_cmd)
        .after(after_cmd)
        .normal_message(normal_message);

    //if has_external_command("ffmpeg") {
    //    framework.group_add(&MUSIC_GROUP);
    //}

    framework
}

#[hook]
async fn before_cmd(ctx: &mut Context, msg: &Message, cmd_name: &str) -> bool {
    info!("Found command {}", cmd_name.bold().underlined());

    if !TYPING_LIST.contains(&cmd_name) {
        msg.channel_id.broadcast_typing(ctx).await.ok();
    }

    EXECUTION_LIST.insert(msg.id, Utc::now());
    true
}

#[hook]
async fn after_cmd(ctx: &mut Context, msg: &Message, cmd: &str, err: CommandResult) {
    let start_time = match EXECUTION_LIST.remove(&msg.id) {
        Some((_, v)) => v.timestamp_millis(),
        None => msg.timestamp.timestamp_millis(),
    };

    match err {
        Ok(_) => {
            let now = Utc::now().timestamp_millis();
            info!(
                "Successfully executed the command {}, time passed {}ms",
                cmd.cyan(),
                now - start_time
            );
        }
        Err(why) => {
            error!("Couldn't execute the command {}\n{:#?}", cmd.magenta(), why);
            let config = crate::read_config().await;
            let color = config.color.error;
            drop(config);

            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|embed| {
                        embed.color(color).description({
                            format!("Cannot execute the command **__{}__**```{}```", cmd, why.0)
                        })
                    })
                })
                .await
                .ok();
        }
    }

    if let Some(info) = ctx.data.read().await.get::<InforKey>() {
        info.executed_one();
    }
}

#[hook]
async fn normal_message(ctx: &mut Context, msg: &Message) {
    macro_rules! exec_func {
        ( $( $x:ident ),* ) => {
            let config = crate::read_config().await;

            $(
                let func = SmallString::from(stringify!($x));
                let mut futs = Vec::new();

                if !config.disable_auto_cmd.contains(&func) {
                    futs.push($x(&ctx, &msg));
                }
            )*

            futures::future::join_all(futs)
                .await
                .into_iter()
                .filter_map(|v| v.err())
                .for_each(|err| println!("Cannot exec an auto process\n{:#?}", err));
        };
    }

    exec_func! {
        mention_rgb,
        repeat_words,
        respect,
        eliza_response,
        rgb_tu,
        find_sauce,
        find_sadkaede
    }
}

fn framwork_config(config: &mut Configuration) -> &mut Configuration {
    let mut owners = HashSet::new();
    let mut disabled_commands = HashSet::new();

    owners.insert(UserId(239825449637642240));

    if !has_external_command("ffmpeg") {
        disabled_commands.insert(String::from("touhou_music_quiz"));
    }

    if !has_external_command("youtube-dl") {
        disabled_commands.insert(String::from("play"));
    }

    config
        .owners(owners)
        .disabled_commands(disabled_commands)
        .case_insensitivity(true)
        .by_space(false)
        .no_dm_prefix(true)
        .dynamic_prefixes([normal_prefix, master_prefix].iter().map(|&v| v as _))
    // .dynamic_prefixes(&[normal_prefix, master_prefix]) doesn't work
}

#[hook]
async fn normal_prefix(_ctx: &mut Context, msg: &Message) -> Option<String> {
    let config = crate::read_config().await;

    msg.guild_id
        .and_then(|guild| config.guilds.get(&guild))
        .and_then(|guild| guild.prefix.to_owned())
        .or_else(|| Some(config.prefix.to_owned()))
        .map(|v| v.to_string())
}

#[hook]
async fn master_prefix(_ctx: &mut Context, msg: &Message) -> Option<String> {
    let config = crate::read_config().await;

    config
        .masters
        .iter()
        .any(|&v| v == msg.author.id)
        .then(|| config.master_prefix.to_string())
}

async fn mention_rgb(ctx: &Context, msg: &Message) -> Result<()> {
    let guild_id = match msg.guild_id {
        Some(v) => v,
        None => return Ok(()),
    };

    if !msg.content.contains("@rgb") {
        return Ok(());
    }

    let to_say = crate::read_config()
        .await
        .guilds
        .get(&guild_id)
        .as_deref()
        .and_then(|v| v.rgblized.as_ref())
        .map(|v| {
            let mut res = String::with_capacity(v.len() * 22);
            for role in v {
                write!(&mut res, "<@&{}>", role.id).unwrap();
            }
            res
        });

    if let Some(m) = to_say {
        for mess in m.split_at_limit(1980, ">") {
            msg.channel_id.say(&ctx, mess).await?;
        }
    }

    Ok(())
}

async fn repeat_words(ctx: &Context, msg: &Message) -> Result<()> {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(()),
    };

    let config = crate::read_config().await;
    let guild = match config.guilds.get(&guild_id) {
        Some(d) => d,
        None => return Ok(()),
    };

    if guild.repeat_words.enable && !guild.repeat_words.words.is_empty() {
        let mess: String = msg
            .content
            .split_whitespace()
            .filter(|v| guild.repeat_words.words.contains(&v.to_lowercase()))
            .map(|v| format!("**{}**", v))
            .join(", ");

        if !mess.is_empty() {
            msg.channel_id.say(ctx, mess).await?;
        }
    }

    Ok(())
}

async fn respect(ctx: &Context, msg: &Message) -> Result<()> {
    if msg
        .content
        .split_whitespace()
        .next()
        .filter(|&v| v == "f" || v == "F")
        .is_none()
    {
        return Ok(());
    }

    let mut content = format!("**{}** has paid their respects", msg.author.name);

    if msg.content.len() > 2 {
        let arg = remove_emote(&msg.content[2..]);
        if !arg.is_empty() {
            write!(&mut content, " for **{}**", arg.trim())?;
        }
    }

    content.push('.');

    let emoji = match crate::read_config().await.respect_emoji {
        None => ReactionType::from('🇫'),
        Some(id) => ReactionType::from(EmojiIdentifier {
            id,
            name: "f_".to_string(),
        }),
    };

    msg.channel_id
        .send_message(ctx, |message| {
            message.content(content);
            message.reactions(Some(emoji));
            message
        })
        .await?;

    Ok(())
}

async fn eliza_response(ctx: &Context, msg: &Message) -> Result<()> {
    let data = ctx.data.read().await;
    let me = data.get::<InforKey>().unwrap();

    if msg.mentions_user_id(me.user_id) {
        let input = remove_mention(&msg.content);
        let response = get_data::<AIStore>(&ctx)
            .await
            .expect("Expected brain in ShareMap.")
            .lock()
            .await
            .respond(&input);

        drop(data);

        msg.channel_id.say(&ctx.http, response).await?;
    }

    Ok(())
}

async fn rgb_tu(ctx: &Context, msg: &Message) -> Result<()> {
    use rand::prelude::*;

    let config = crate::read_config().await;
    let rgb = match config.rgb.as_ref() {
        Some(r) => r,
        None => return Ok(()),
    };

    if msg.guild_id.is_some()
        && msg.author.id.0 == 314444746959355905
        && msg
            .content
            .to_lowercase()
            .split_whitespace()
            .map(SmallString::from)
            .any(|v| rgb.tu.contains(&v))
    {
        let mut rng = SmallRng::from_entropy();
        let num = rng.gen::<f32>();

        msg
        .channel_id
        .send_message(&ctx, |m| m.embed(|embed| {
            embed
            .color((num * 16777215_f32) as u32)
            .image("https://cdn.discordapp.com/attachments/418811018698031107/661658331613495297/2019-09-15_220414.png")
        }))
        .await?;

        if num < 0.05 {
            use tokio::fs;
            use tokio::stream::StreamExt;

            let path = &rgb.evidence;

            let evi = fs::read_dir(path)
                .await?
                .filter_map(|v| v.ok())
                .collect::<Vec<_>>()
                .await
                .choose(&mut rng)
                .map(|v| v.path());

            if let Some(evi) = evi {
                msg.channel_id
                    .send_message(&ctx, |m| m.add_file(&evi))
                    .await?;
            }
        }
    }

    Ok(())
}

async fn find_sauce(ctx: &Context, msg: &Message) -> Result<()> {
    use futures::future::{self, FutureExt};
    use magic::sauce::get_sauce;

    let config = crate::read_config().await;

    let is_watching_channel = msg
        .guild_id
        .and_then(|v| config.guilds.get(&v))
        .filter(|v| v.find_sauce.enable)
        .filter(|v| v.find_sauce.all || v.find_sauce.channels.contains(&msg.channel_id))
        .is_some();

    if !is_watching_channel || msg.is_own(&ctx).await {
        return Ok(());
    }

    let timeout = Duration::from_secs(config.sauce.wait_duration as u64);
    let emoji_id = match config.sauce.emoji {
        Some(e) => e,
        None => return Ok(()),
    };

    drop(config);

    let sauces = msg
        .attachments
        .iter()
        .filter(|v| v.width.is_some())
        .filter(|v| !v.url.ends_with(".gif"))
        .map(|v| async move { get_sauce(&v.url, None).await });

    let sauces: Vec<_> = future::join_all(sauces)
        .await
        .into_iter()
        .filter_map(|v| v.ok())
        .filter(|v| v.found())
        .collect();

    if sauces.is_empty() {
        return Ok(());
    }

    let reaction = EmojiIdentifier {
        id: emoji_id,
        name: String::from("sauce"),
    };

    msg.react(ctx, reaction).await?;
    let collector = msg
        .await_reaction(&ctx)
        .timeout(timeout)
        .filter(move |v| matches!(v.emoji, ReactionType::Custom{ id, .. } if id == emoji_id))
        .removed(false)
        .await;

    if let Some(reaction) = collector {
        let http = std::sync::Arc::clone(&ctx.http);
        tokio::spawn(async move { reaction.as_inner_ref().delete(http).await.ok() });
        for sauce in sauces {
            msg.channel_id
                .send_message(&ctx, |m| {
                    m.embed(|mut embed| {
                        sauce.to_embed(&mut embed);
                        embed
                    })
                })
                .await?;
        }
    }

    Ok(())
}

// Simply a clone of the find_sauce due to similar functionality
async fn find_sadkaede(ctx: &Context, msg: &Message) -> Result<()> {
    if msg.content.len() < 20 {
        return Ok(());
    }

    let config = crate::read_config().await;

    let is_watching_channel = msg
        .guild_id
        .and_then(|v| config.guilds.get(&v))
        .filter(|v| v.find_sadkaede.enable)
        .filter(|v| v.find_sadkaede.all || v.find_sadkaede.channels.contains(&msg.channel_id))
        .is_some();

    if !is_watching_channel || msg.is_own(&ctx).await {
        return Ok(());
    }

    let gids = parse_eh_token(&msg.content);
    if gids.is_empty() {
        return Ok(());
    }

    let emoji_id = match config.sadkaede.emoji {
        Some(e) => e,
        None => return Ok(()),
    };

    let timeout = Duration::from_secs(config.sadkaede.wait_duration as u64);

    drop(config);

    let data = get_data::<ReqwestClient>(&ctx)
        .await
        .unwrap()
        .gmetadata(gids.into_iter().take(25))
        .await?;

    let is_channel_nsfw = is_nsfw_channel(&ctx, msg.channel_id).await;
    let data: Vec<_> = data
        .into_iter()
        .filter(|data| is_channel_nsfw || data.is_sfw())
        .collect();

    if data.is_empty() {
        return Ok(());
    }

    let reaction = EmojiIdentifier {
        id: emoji_id,
        name: String::from("sadkaede"),
    };

    msg.react(ctx, reaction).await?;
    let collector = msg
        .await_reaction(&ctx)
        .timeout(timeout)
        .filter(move |v| matches!(v.emoji, ReactionType::Custom{ id, .. } if id == emoji_id))
        .removed(false)
        .await;

    if let Some(reaction) = collector {
        let http = std::sync::Arc::clone(&ctx.http);
        tokio::spawn(async move { reaction.as_inner_ref().delete(http).await.ok() });
        for sadkaede in data {
            msg.channel_id
                .send_message(&ctx, |m| {
                    m.embed(|mut embed| {
                        sadkaede.to_embed(&mut embed);
                        embed
                    })
                })
                .await?;
        }
    }

    Ok(())
}

// struct SchedulerReact {
//     data: Vec<Box<dyn ToEmbed>>,
//     channel_id: ChannelId,
//     timer: JobHandle,
//     emoji_id: EmojiId,
// }
//
// lazy_static! {
//     static ref WATCHING_REACT: Mutex<HashMap<MessageId, SchedulerReact>> = Default::default();
// }
//
// #[rustfmt::skip]
// fn find_sauce(ctx: &Context, msg: &Message) {
//     use magic::sauce::get_sauce;
//
//     let config = crate::read_config();
//
//     let emoji_id = match config.sauce.emoji {
//         Some(e) => e,
//         None => return
//     };
//
//     let is_watching_channel = msg
//         .guild_id
//         .and_then(|v| config.guilds.get(&v))
//         .filter(|v| v.find_sauce.enable)
//         .filter(|v| v.find_sauce.all || v.find_sauce.channels.contains(&msg.channel_id))
//         .is_some();
//
//     if !is_watching_channel || msg.is_own(&ctx) {
//         return;
//     }
//
//     let sauces: Vec<_> = msg
//         .attachments
//         .iter()
//         .filter(|v| v.width.is_some())
//         .filter(|v| !v.url.ends_with(".gif"))
//         .filter_map(|v| get_sauce(&v.url, None).ok())
//         .filter(|v| v.found())
//         .map(|v| Box::new(v) as Box<_>)
//         .collect();
//
//     if sauces.is_empty() {
//         return
//     }
//
//     let reaction = EmojiIdentifier {
//         id: emoji_id,
//         name: String::from("sauce"),
//     };
//
//     if let Err(why) = msg.react(ctx, reaction.clone()) {
//         error!("Cannot react for the sauce\n{:#?}", why);
//         return
//     }
//
//     let http = ctx.http.clone();
//     let channel_id = msg.channel_id.0;
//     let msg_id = msg.id.0;
//     let duration = Duration::from_secs(config.sauce.wait_duration as u64);
//
//     drop(config);
//
//     let timer = crate::global::GLOBAL_POOL.execute_after(duration, move || {
//         let emoji = reaction.into();
//         if let Err(why) = http.delete_reaction(channel_id, msg_id, None, &emoji) {
//             error!("Cannot delete the sauce reaction\n{:#?}", why);
//         }
//     });
//
//     let scheduler = SchedulerReact {
//         timer,
//         data: sauces,
//         channel_id: msg.channel_id,
//         emoji_id,
//     };
//
//     let mut data_r = WATCHING_REACT.lock();
//
//     data_r.insert(msg.id, scheduler);
//     if data_r.len() == 1 {
//         get_data::<CustomEventList>(&ctx).unwrap().add("WatchingEmo", watch_emo_event);
//     }
// }
//
// // Simply a clone of the find_sauce due to similar functionality
// async fn find_sadkaede(ctx: &Context, msg: &Message) {
//     if msg.content.len() < 20 {
//         return;
//     }
//
//     let config = crate::read_config().await;
//
//     let emoji_id = match config.sadkaede.emoji {
//         Some(e) => e,
//         None => return,
//     };
//
//     let is_watching_channel = msg
//         .guild_id
//         .and_then(|v| config.guilds.get(&v))
//         .filter(|v| v.find_sadkaede.enable)
//         .filter(|v| v.find_sadkaede.all || v.find_sadkaede.channels.contains(&msg.channel_id))
//         .is_some();
//
//     if !is_watching_channel || msg.is_own(&ctx) {
//         return;
//     }
//
//     let req = get_data::<ReqwestClient>(&ctx).unwrap();
//     let gids = parse_eh_token(&msg.content);
//
//     if gids.is_empty() {
//         return;
//     }
//
//     let data = match req.gmetadata(gids.into_iter().take(25)).await {
//         Ok(d) => d,
//         Err(_) => return,
//     };
//
//     let is_channel_nsfw = is_nsfw_channel(&ctx, msg.channel_id);
//
//     let data: Vec<_> = data
//         .into_iter()
//         .filter(|data| is_channel_nsfw || data.is_sfw())
//         .map(|v| Box::new(v) as Box<_>)
//         .collect();
//
//     if data.is_empty() {
//         return;
//     }
//
//     let reaction = EmojiIdentifier {
//         id: emoji_id,
//         name: String::from("sadkaede"),
//     };
//
//     if let Err(why) = msg.react(ctx, reaction.clone()) {
//         error!("Cannot reaction to the sadkaede\n{:#?}", why);
//         return;
//     }
//
//     let http = ctx.http.clone();
//     let channel_id = msg.channel_id.0;
//     let msg_id = msg.id.0;
//     let duration = Duration::from_secs(config.sadkaede.wait_duration as u64);
//
//     drop(config);
//
//     let timer = crate::global::GLOBAL_POOL.execute_after(duration, move || {
//         let emoji = reaction.into();
//         if let Err(why) = http.delete_reaction(channel_id, msg_id, None, &emoji) {
//             error!("Cannot delete the sadkaede reaction\n{:#?}", why);
//         }
//     });
//
//     let scheduler = SchedulerReact {
//         timer,
//         data,
//         channel_id: msg.channel_id,
//         emoji_id,
//     };
//
//     let mut data_r = WATCHING_REACT.lock();
//
//     data_r.insert(msg.id, scheduler);
//     if data_r.len() == 1 {
//         get_data::<CustomEventList>(&ctx)
//             .unwrap()
//             .add("WatchingEmo", watch_emo_event);
//     }
// }
//
// fn watch_emo_event(ctx: &Context, ev: &Event) {
//     fn process_delete_message(ctx: &Context, id: MessageId) {
//         let mut data_r = WATCHING_REACT.lock();
//         if let Some(s) = data_r.remove(&id) {
//             s.timer.cancel();
//             if data_r.is_empty() {
//                 get_data::<CustomEventList>(ctx)
//                     .unwrap()
//                     .done("WatchingEmo");
//             }
//         }
//     }
//
//     match ev {
//         Event::MessageDelete(e) => process_delete_message(&ctx, e.message_id),
//         Event::MessageDeleteBulk(e) => {
//             for id in &e.ids {
//                 process_delete_message(&ctx, *id);
//             }
//         }
//
//         Event::ReactionAdd(reaction) => {
//             let reaction = &reaction.reaction;
//
//             if reaction.user_id == ctx.data.read().get::<InforKey>().unwrap().user_id {
//                 return;
//             }
//
//             let mut data_r = WATCHING_REACT.lock();
//
//             let watching = data_r
//                 .get(&reaction.message_id)
//                 .filter(|v| {
//                     if let ReactionType::Custom { id, .. } = reaction.emoji {
//                         v.emoji_id == id
//                     } else {
//                         false
//                     }
//                 })
//                 .is_some();
//
//             if !watching {
//                 return;
//             }
//
//             if let Some(s) = data_r.remove(&reaction.message_id) {
//                 s.timer.cancel();
//                 reaction.delete(ctx).ok();
//
//                 for d in s.data {
//                     let sent = s.channel_id.send_message(&ctx, |m| {
//                         m.embed(|mut embed| {
//                             d.to_embed(&mut embed);
//                             embed
//                         })
//                     });
//
//                     if let Err(why) = sent {
//                         error!("Cannot send the sauce\n{:#?}", why);
//                     }
//                 }
//
//                 if data_r.is_empty() {
//                     get_data::<CustomEventList>(&ctx)
//                         .unwrap()
//                         .done("WatchingEmo");
//                 }
//             }
//         }
//
//         _ => {}
//     }
// }
//
