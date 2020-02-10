use serenity::{
    client::Context,
    framework::{
        standard::{
            help_commands, macros::help, Args, CommandGroup, CommandResult, Configuration,
            HelpOptions,
        },
        Framework, StandardFramework,
    },
    model::{
        channel::{Message, ReactionType},
        event::Event,
        id::{ChannelId, MessageId, UserId},
        misc::EmojiIdentifier,
    },
};

use crate::{
    commands::*,
    constants::{SAUCE_EMOJI, SAUCE_WAIT_DURATION},
    storages::{AIStore, CustomEventList, InforKey, MasterList},
    traits::ToEmbed,
    utils::*,
};

use chrono::{DateTime, Utc};
use colorful::Colorful;
use dashmap::DashMap;
use lazy_static::lazy_static;
use log::{error, info};
use magic::has_external_command;
use magic::sauce::SauceNao;
use magic::traits::MagicIter as _;
use parking_lot::Mutex;
use scheduled_thread_pool::JobHandle;
use std::collections::{HashMap, HashSet};

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
fn stolen_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

pub fn get_framework() -> impl Framework {
    let mut framework = StandardFramework::new()
        .bucket("basic", |b| b.delay(2).time_span(10).limit(3))
        .group(&MASTER_GROUP)
        .group(&GUILDMASTER_GROUP)
        .group(&ADMINISTRATION_GROUP)
        .group(&GENERAL_GROUP)
        .group(&GAME_GROUP)
        .group(&POKEMON_GROUP)
        // .group(&UTILITY_GROUP)
        .group(&IMAGE_GROUP)
        .group(&RGB_GROUP)
        .help(&STOLEN_HELP)
        .configure(framwork_config)
        .before(before_cmd)
        .after(after_cmd)
        .normal_message(normal_message);

    if has_external_command("ffmpeg") {
        framework.group_add(&MUSIC_GROUP);
    }

    framework
}

fn before_cmd(_ctx: &mut Context, msg: &Message, cmd_name: &str) -> bool {
    info!("Found command {}", cmd_name.bold().underlined());
    EXECUTION_LIST.insert(msg.id, Utc::now());
    true
}

fn after_cmd(ctx: &mut Context, msg: &Message, cmd: &str, err: CommandResult) {
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
            send_embed(&ctx.http, msg.channel_id, |embed| {
                let config = crate::read_config();
                embed.color(config.color.error).description({
                    format!("Cannot execute the command **__{}__**```{}```", cmd, why.0)
                })
            });
        }
    }

    ctx.data.read().get::<InforKey>().unwrap().executed_one();
}

fn normal_message(ctx: &mut Context, msg: &Message) {
    macro_rules! exec_func {
        ( $( $x:ident ),* ) => {
            $(
                $x(&ctx, &msg);
            )*
        };
    }

    exec_func! {
        mention_rgb,
        repeat_words,
        respect,
        eliza_response,
        rgb_tu,
        find_sauce
    }
}

fn framwork_config<'a>(config: &'a mut Configuration) -> &'a mut Configuration {
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
        .dynamic_prefixes(&[normal_prefix, master_prefix])
}

fn normal_prefix(_ctx: &mut Context, msg: &Message) -> Option<String> {
    let config = crate::read_config();

    msg.guild_id
        .and_then(|guild| config.guilds.get(&guild))
        .and_then(|guild| guild.prefix.to_owned())
        .or_else(|| Some(config.prefix.to_owned()))
}

fn master_prefix(ctx: &mut Context, msg: &Message) -> Option<String> {
    get_data::<MasterList>(&ctx).and_then(|v| {
        v.read()
            .iter()
            .find(|&&v| v == msg.author.id)
            .map(|_| "%".to_string())
    })
}

fn mention_rgb(ctx: &Context, msg: &Message) {
    let guild_id = match msg.guild_id {
        Some(v) => v,
        None => return,
    };

    if !msg.content.contains("@rgb") {
        return;
    }

    let to_say = crate::read_config().guilds.get(&guild_id).and_then(|v| {
        v.rgblized.as_ref().map(|v| {
            v.iter()
                .map(|x| format!("<@&{}>", x.id))
                .collect::<String>()
        })
    });

    if let Some(m) = to_say {
        split_message(&m, 1980, ">")
            .into_iter()
            .filter_map(|s| msg.channel_id.say(&ctx, s).err())
            .for_each(|err| error!("Cannot mention rgb\n{}", err))
    }
}

fn repeat_words(ctx: &Context, msg: &Message) {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return,
    };

    let config = crate::read_config();
    let guild = match config.guilds.get(&guild_id) {
        Some(d) => d,
        None => return,
    };

    if guild.repeat_words.enable && !guild.repeat_words.words.is_empty() {
        let mess: String = msg
            .content
            .split_whitespace()
            .filter(|v| guild.repeat_words.words.contains(&v.to_lowercase()))
            .map(|v| format!("**{}**", v))
            .join(", ");

        if !mess.is_empty() {
            if let Err(why) = msg.channel_id.say(ctx, mess) {
                error!("Error occur while repeating words:\n{}", why)
            }
        }
    }
}

fn respect(ctx: &Context, msg: &Message) {
    if msg
        .content
        .split_whitespace()
        .next()
        .filter(|&v| v == "f" || v == "F")
        .is_none()
    {
        return;
    }

    let mut message = format!("**{}** has paid their respects", msg.author.name);

    if msg.content.len() > 2 {
        let content = remove_emote(&msg.content[2..]);
        if !content.is_empty() {
            message.push_str(&format!(" for **{}**", content.trim()));
        }
    }

    message.push('.');

    if let Err(why) = msg.channel_id.say(ctx, message) {
        error!("Cannot pay respect:\n{:#?}", why);
    }
}

fn eliza_response(ctx: &Context, msg: &Message) {
    let data = ctx.data.read();
    let me = data.get::<InforKey>().unwrap();

    if msg.mentions_user_id(me.user_id) {
        let brain = get_data::<AIStore>(&ctx).expect("Expected brain in ShareMap.");

        let input = remove_mention(&msg.content);
        let mut eliza = brain.lock();
        let response = eliza.respond(&input);

        if let Err(err) = msg.channel_id.say(&ctx.http, response) {
            error!("Cannot send the smart response\n{:#?}", err);
        }
    }
}

fn rgb_tu(ctx: &Context, msg: &Message) {
    use rand::prelude::*;
    use std::fs;
    
    let config = crate::read_config();
    let rgb = match config.rgb.as_ref() {
        Some(r) => r,
        None => return
    };

    if msg.guild_id.is_some()
        && msg.author.id.0 == 314444746959355905
        && msg
            .content
            .to_lowercase()
            .split_whitespace()
            .find(|v| rgb.tu.contains(&v.to_string()))
            .is_some()
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
        .ok();

        if num < 0.05 {
            let path = &rgb.evidence;

            let evi = fs::read_dir(path)
                .unwrap()
                .filter_map(|v| v.ok())
                .choose(&mut SmallRng::from_entropy())
                .unwrap()
                .path();

            msg.channel_id.send_message(&ctx, |m| m.add_file(&evi)).ok();
        }
    }
}

struct SchedulerSauce {
    sauces: Vec<SauceNao>,
    channel_id: ChannelId,
    timer: JobHandle,
}

lazy_static! {
    // The emoji :sauce: is from an private guild,
    // that means only the bot can create this reaction
    // That's why we don't have to check it first
    // Mutex here because we always need its write access
    // Consider using RwLock or DashMap when the bot grow
    static ref SAUCE_REACT: Mutex<HashMap<MessageId, SchedulerSauce>> = Default::default();
}

#[rustfmt::skip]
fn find_sauce(ctx: &Context, msg: &Message) {
    use magic::sauce::get_sauce;

    let config = crate::read_config();
    
    let is_watching_channel = msg
        .guild_id
        .and_then(|v| config.guilds.get(&v))
        .filter(|v| v.find_sauce.enable)
        .filter(|v| v.find_sauce.all || v.find_sauce.channels.contains(&msg.channel_id))
        .is_some();
        
    if !is_watching_channel || msg.is_own(&ctx) {
        return;
    }
    
    let sauces: Vec<SauceNao> = msg
        .attachments
        .iter()
        .filter(|v| v.width.is_some())
        .filter(|v| !v.url.ends_with(".gif"))
        .filter_map(|v| get_sauce(&v.url, None).ok())
        .filter(|v| v.found())
        .filter(|_| msg.react(ctx, _sauce_emoji()).is_ok())
        .collect();
        
    if sauces.is_empty() {
        return
    }
    
    let http = ctx.http.clone();
    let channel_id = msg.channel_id.0;
    let msg_id = msg.id.0;
    
    let timer = crate::global::GLOBAL_POOL.execute_after(SAUCE_WAIT_DURATION, move || {
        let emoji = _sauce_emoji().into();
        if let Err(why) = http.delete_reaction(channel_id, msg_id, None, &emoji) {
            error!("Cannot delete the sauce reaction\n{:#?}", why);
        }
    });
    
    let scheduler = SchedulerSauce {
        timer,
        sauces,
        channel_id: msg.channel_id,
    };
    
    let mut sauce_r = SAUCE_REACT.lock();
    
    sauce_r.insert(msg.id, scheduler);
    if sauce_r.len() == 1 {
        get_data::<CustomEventList>(&ctx).unwrap().add("WatchingSauce", sauce_event);
    }
}

fn sauce_event(ctx: &Context, ev: &Event) {
    fn process_delete_message(ctx: &Context, id: &MessageId) {
        let mut sauce_r = SAUCE_REACT.lock();
        if let Some(s) = sauce_r.remove(id) {
            s.timer.cancel();
            if sauce_r.is_empty() {
                get_data::<CustomEventList>(ctx)
                    .unwrap()
                    .done("WatchingSauce");
            }
        }
    }

    match ev {
        Event::MessageDelete(e) => process_delete_message(&ctx, &e.message_id),
        Event::MessageDeleteBulk(e) => {
            for id in &e.ids {
                process_delete_message(&ctx, id);
            }
        }

        Event::ReactionAdd(reaction) => {
            let reaction = &reaction.reaction;
            if let ReactionType::Custom { id, .. } = reaction.emoji {
                if id != SAUCE_EMOJI {
                    return;
                }
            }

            if reaction.user_id == ctx.data.read().get::<InforKey>().unwrap().user_id {
                return;
            }

            let mut sauce_r = SAUCE_REACT.lock();

            if let Some(s) = sauce_r.remove(&reaction.message_id) {
                s.timer.cancel();
                reaction.delete(ctx).ok();

                for sauce in s.sauces {
                    let sent = s.channel_id.send_message(&ctx, |m| {
                        m.embed(|mut embed| {
                            sauce.to_embed(&mut embed);
                            embed
                        })
                    });

                    if let Err(why) = sent {
                        error!("Cannot send the sauce\n{:#?}", why);
                    }
                }

                if sauce_r.is_empty() {
                    get_data::<CustomEventList>(&ctx)
                        .unwrap()
                        .done("WatchingSauce");
                }
            }
        }

        _ => {}
    }
}

#[inline]
fn _sauce_emoji() -> EmojiIdentifier {
    EmojiIdentifier {
        id: SAUCE_EMOJI,
        name: "sauce".to_string(),
    }
}
