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
    event::Event,
    id::{ChannelId, EmojiId, MessageId, UserId},
    misc::EmojiIdentifier,
};

use crate::{
    commands::*,
    storages::{AIStore, CustomEventList, InforKey, ReqwestClient},
    traits::ToEmbed,
    utils::*,
};

use chrono::{DateTime, Utc};
use colorful::Colorful;
use core::time::Duration;
use dashmap::DashMap;
use lazy_static::lazy_static;
use log::{error, info};
use magic::has_external_command;
use magic::sauce::SauceNao;
use requester::ehentai::{EhentaiApi, Gmetadata};
use scheduled_thread_pool::JobHandle;
use smallstr::SmallString;
use std::collections::{HashMap, HashSet};
use tokio::sync::Mutex;

use magic::traits::MagicBool as _;
use magic::traits::MagicIter as _;
use magic::traits::MagicStr as _;
use std::fmt::Write as _;

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
        // .group(&MASTER_GROUP)
        // .group(&GUILDMASTER_GROUP)
        .group(&ADMINISTRATION_GROUP)
        // .group(&CORONA_GROUP)
        .group(&GENERAL_GROUP)
        // .group(&GAME_GROUP)
        // .group(&POKEMON_GROUP)
        // .group(&UTILITY_GROUP)
        // .group(&IMAGE_GROUP)
        // .group(&RGB_GROUP)
        // .help(&STOLEN_HELP)
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
async fn before_cmd(_ctx: &mut Context, msg: &Message, cmd_name: &str) -> bool {
    info!("Found command {}", cmd_name.bold().underlined());
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

            msg.channel_id.send_message(&ctx.http, |m| m.embed(|embed| {
                embed.color(color).description({
                    format!("Cannot execute the command **__{}__**```{}```", cmd, why.0)
                })
            })).await.ok();
        }
    }

    if let Some(info) = ctx.data.read().await.get::<InforKey>() {
        info.executed_one();
    }
}

#[hook]
async fn normal_message(ctx: &mut Context, msg: &Message) {
    // macro_rules! exec_func {
    //     ( $( $x:ident ),* ) => {
    //         let config = crate::read_config();

    //         $(
    //             if !config.disable_auto_cmd.contains(&SmallString::from(stringify!($x))) {
    //                 $x(&ctx, &msg);
    //             }
    //         )*
    //     };
    // }

    // exec_func! {
    //     mention_rgb,
    //     repeat_words,
    //     respect,
    //     eliza_response,
    //     rgb_tu,
    //     find_sauce,
    //     find_sadkaede
    // }
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

// fn mention_rgb(ctx: &Context, msg: &Message) {
//     let guild_id = match msg.guild_id {
//         Some(v) => v,
//         None => return,
//     };
//
//     if !msg.content.contains("@rgb") {
//         return;
//     }
//
//     let to_say = crate::read_config()
//         .guilds
//         .get(&guild_id)
//         .as_deref()
//         .and_then(|v| v.rgblized.as_ref())
//         .map(|v| {
//             let mut res = String::with_capacity(v.len() * 22);
//             for role in v {
//                 write!(&mut res, "<@&{}>", role.id).unwrap();
//             }
//             res
//         });
//
//     if let Some(m) = to_say {
//         m.split_at_limit(1980, ">")
//             .filter_map(|s| msg.channel_id.say(&ctx, s).err())
//             .for_each(|err| error!("Cannot mention rgb\n{}", err))
//     }
// }
//
// fn repeat_words(ctx: &Context, msg: &Message) {
//     let guild_id = match msg.guild_id {
//         Some(g) => g,
//         None => return,
//     };
//
//     let config = crate::read_config();
//     let guild = match config.guilds.get(&guild_id) {
//         Some(d) => d,
//         None => return,
//     };
//
//     if guild.repeat_words.enable && !guild.repeat_words.words.is_empty() {
//         let mess: String = msg
//             .content
//             .split_whitespace()
//             .filter(|v| guild.repeat_words.words.contains(&v.to_lowercase()))
//             .map(|v| format!("**{}**", v))
//             .join(", ");
//
//         if !mess.is_empty() {
//             if let Err(why) = msg.channel_id.say(ctx, mess) {
//                 error!("Error occur while repeating words:\n{}", why)
//             }
//         }
//     }
// }
//
// fn respect(ctx: &Context, msg: &Message) {
//     if msg
//         .content
//         .split_whitespace()
//         .next()
//         .filter(|&v| v == "f" || v == "F")
//         .is_none()
//     {
//         return;
//     }
//
//     let mut content = format!("**{}** has paid their respects", msg.author.name);
//
//     if msg.content.len() > 2 {
//         let arg = remove_emote(&msg.content[2..]);
//         if !arg.is_empty() {
//             write!(&mut content, " for **{}**", arg.trim()).unwrap();
//         }
//     }
//
//     content.push('.');
//
//     let emoji = match crate::read_config().respect_emoji {
//         None => ReactionType::from('🇫'),
//         Some(id) => ReactionType::from(EmojiIdentifier {
//             id,
//             name: "f_".to_string(),
//         }),
//     };
//
//     let send = msg.channel_id.send_message(ctx, |message| {
//         message.content(content);
//         message.reactions(Some(emoji));
//         message
//     });
//
//     if let Err(why) = send {
//         error!("Cannot pay respect:\n{:#?}", why);
//     }
// }
//
// fn eliza_response(ctx: &Context, msg: &Message) {
//     let data = ctx.data.read();
//     let me = data.get::<InforKey>().unwrap();
//
//     if msg.mentions_user_id(me.user_id) {
//         let brain = get_data::<AIStore>(&ctx).expect("Expected brain in ShareMap.");
//
//         let input = remove_mention(&msg.content);
//         let mut eliza = brain.lock();
//         let response = eliza.respond(&input);
//
//         if let Err(err) = msg.channel_id.say(&ctx.http, response) {
//             error!("Cannot send the smart response\n{:#?}", err);
//         }
//     }
// }
//
// fn rgb_tu(ctx: &Context, msg: &Message) {
//     use rand::prelude::*;
//     use std::fs;
//
//     let config = crate::read_config();
//     let rgb = match config.rgb.as_ref() {
//         Some(r) => r,
//         None => return,
//     };
//
//     if msg.guild_id.is_some()
//         && msg.author.id.0 == 314444746959355905
//         && msg
//             .content
//             .to_lowercase()
//             .split_whitespace()
//             .map(SmallString::from)
//             .any(|v| rgb.tu.contains(&v))
//     {
//         let mut rng = SmallRng::from_entropy();
//         let num = rng.gen::<f32>();
//
//         msg
//         .channel_id
//         .send_message(&ctx, |m| m.embed(|embed| {
//             embed
//             .color((num * 16777215_f32) as u32)
//             .image("https://cdn.discordapp.com/attachments/418811018698031107/661658331613495297/2019-09-15_220414.png")
//         }))
//         .ok();
//
//         if num < 0.05 {
//             let path = &rgb.evidence;
//
//             let evi = fs::read_dir(path)
//                 .unwrap()
//                 .filter_map(|v| v.ok())
//                 .choose(&mut SmallRng::from_entropy())
//                 .unwrap()
//                 .path();
//
//             msg.channel_id.send_message(&ctx, |m| m.add_file(&evi)).ok();
//         }
//     }
// }
//
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
