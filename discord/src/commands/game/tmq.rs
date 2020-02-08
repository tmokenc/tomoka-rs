use crate::commands::prelude::*;
use crate::storages::CustomEventList;
use crate::Result;
use colorful::Colorful;
use lazy_static::lazy_static;
use log::{error, info};
use mp3_duration;
use rand::prelude::*;
use rand::thread_rng;
use regex::Regex;
use serenity::model::event::Event;
use serenity::model::id::{ChannelId, EmojiId, MessageId, UserId};
use serenity::model::misc::EmojiIdentifier;
use serenity::voice::{ffmpeg_optioned, AudioSource, Bitrate};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, thread};

type TmqCollect = Mutex<HashMap<ChannelId, HashMap<UserId, TmqCollector>>>;

lazy_static! {
    static ref TOUHOU_VERSION: HashMap<String, u64> = {
        let file = File::open("tmq_emo.json").unwrap();
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).unwrap()
    };
    static ref TMQ_COLLECTOR: TmqCollect = Mutex::new(HashMap::new());
}

#[derive(Debug)]
struct TmqCollector {
    message_id: MessageId,
    answer: String,
}

#[command]
#[owners_only]
#[only_in(guilds)]
#[aliases("tmq", "touhoumusicquiz")]
fn touhou_music_quiz(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(()),
    };

    if let Some(channel) = is_playing(&ctx, guild_id) {
        msg.channel_id.say(
            &ctx,
            format!("I'm current playing on channel <#{}>", channel.0),
        )?;
        return Ok(());
    }

    let voice_channel = match get_user_voice_channel(&ctx, guild_id, msg.author.id) {
        Some(c) => c,
        None => return Ok(()),
    };

    let voice_manager = get_voice_manager(&ctx);
    let mut voice = {
        let mut manager = voice_manager.lock();
        match manager.join(guild_id, voice_channel) {
            Some(c) => c.clone(),
            None => return Ok(()),
        }
    };

    voice.set_bitrate(Bitrate::BitsPerSecond(192000));

    let custom_events = ctx
        .data
        .read()
        .get::<CustomEventList>()
        .cloned()
        .expect("Expected CustomEventList");

    {
        let mut collector = TMQ_COLLECTOR.lock();
        if collector.is_empty() {
            custom_events.add("tmq", tmq_event_handler);
        }

        collector.insert(msg.channel_id, HashMap::new());
    }

    loop {
        let (path, version) = get_quiz()?;
        info!("The answer is touhou {}", version.as_str().blue());

        let audio = get_audio(path, TMQ_DURATION)?;
        let _locked_audio = voice.play_only(audio);

        thread::sleep(Duration::from_secs_f32(TMQ_DURATION - 2.0));

        let (winners_list, loosers_list): (Vec<_>, Vec<_>) = TMQ_COLLECTOR
            .lock()
            .get_mut(&msg.channel_id)
            .unwrap()
            .drain()
            .partition(|(_, v)| v.answer == version);
            
        let response = format!(
            "***Time up!!!***\n**The answer is**: Touhou {}\n**Correct**: {}\n**Incorrect**: {}",
            version,
            winners_list.len(),
            loosers_list.len(),
        );

        let winners = winners_list
            .into_iter()
            .map(|v| format!("<@{}>\n", v.0))
            .collect::<String>();

        
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|embed| {
                embed.description(response);
                embed.timestamp(now());
                if !winners.is_empty() {
                    embed.field("Winners", winners, true);
                }
                embed
            })
        })?;

        if is_dead_channel(&ctx, voice_channel) {
            break;
        }
    }

    voice_manager.lock().leave(guild_id);
    msg.channel_id.say(
        &ctx,
        format!(
            "Leaving channel <#{}>, since there are no human left",
            voice_channel
        ),
    )?;

    {
        let mut collector = TMQ_COLLECTOR.lock();
        collector.remove(&msg.channel_id);

        if collector.is_empty() {
            custom_events.done("tmq");
        }
    }

    Ok(())
}

fn get_audio(path: impl AsRef<Path>, duration: f32) -> Result<Box<dyn AudioSource>> {
    let path = path.as_ref();
    let total_time = mp3_duration::from_path(&path)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "quickfix"))?
        .as_secs_f32();

    let start_from = rand::random::<f32>() * total_time - duration;
    let opt = &[
        "-ss",
        &start_from.to_string(),
        "-f",
        "s16le",
        "-ac",
        "1",
        "-ar",
        "48000",
        "-acodec",
        "pcm_s16le",
        "-",
    ];

    let result = ffmpeg_optioned(path, opt)?;
    Ok(result)
}

fn get_quiz() -> Result<(PathBuf, String)> {
    let path = env::var("TMQ_SOURCE")?;
    let mut rng = thread_rng();

    let touhou = fs::read_dir(path)?
        .filter_map(|v| v.ok().map(|v| v.path()).filter(|v| v.is_dir()))
        .choose(&mut rng)
        .unwrap();

    let name = touhou.file_name().unwrap();
    let version = parse_touhou_version(name.to_str().unwrap()).unwrap();

    let file = touhou
        .read_dir()?
        .filter_map(|v| {
            v.ok().filter(|x| {
                let os_name = x.file_name();
                let name = os_name.to_str().unwrap();
                name.ends_with(".mp3") && !name.contains("Player Score")
            })
        })
        .choose(&mut rng)
        .unwrap();

    info!("Choosed song for TMQ: {:?}", &file);

    Ok((file.path(), version.to_string()))
}

fn parse_touhou_version<S: AsRef<str>>(s: S) -> Option<String> {
    lazy_static! {
        static ref TOUHOU_RE: Regex = Regex::new(r"(th|touhou) ?([0-9\.]+)").unwrap();
    }

    let s = s.as_ref().replace("_", ".");

    TOUHOU_RE.captures_iter(&s).find_map(|v| {
        let num = v.get(2).unwrap().as_str();
        let version = num.parse::<f32>().unwrap().to_string();

        TOUHOU_VERSION.get(&version).map(|_| version)
    })
}

#[inline]
fn is_dead_channel(ctx: &Context, channel_id: ChannelId) -> bool {
    channel_id
        .to_channel(ctx)
        .ok()
        .and_then(|v| v.guild())
        .and_then(|v| v.read().members(ctx).ok())
        .map(|v| {
            v.into_iter()
                .filter(|x| !x.user.read().bot)
                .collect::<Vec<_>>()
        })
        .filter(|v| v.is_empty())
        .is_some()
}

fn touhou_emoji(version: &str) -> EmojiIdentifier {
    let id = TOUHOU_VERSION.get(version).unwrap();
    let name = format!("th{}", version.replace(".", "_"));

    EmojiIdentifier {
        id: EmojiId(*id),
        name,
    }
}

fn tmq_event_handler(ctx: &Context, ev: &Event) {
    match ev {
        Event::MessageCreate(event) => {
            let channel_id = event.message.channel_id;
            let author = event.message.author.id;

            if let Some(chan) = TMQ_COLLECTOR.lock().get_mut(&channel_id) {
                if chan.contains_key(&author) {
                    return;
                }

                let touhou_version = parse_touhou_version(&event.message.content);
                if let Some(version) = touhou_version {
                    let emoji = touhou_emoji(&version);
                    let collector = TmqCollector {
                        message_id: event.message.id,
                        answer: version,
                    };

                    chan.insert(author, collector);
                    if let Err(why) = event.message.react(ctx, emoji) {
                        error!("{}", why);
                    };
                }
            }
        }

        Event::MessageUpdate(event) => {
            let channel_id = event.channel_id;

            let content = event.content.as_ref();
            if let (Some(chan), Some(con)) = (TMQ_COLLECTOR.lock().get_mut(&channel_id), content) {
                let ans = chan
                    .iter_mut()
                    .find(|(_, v)| v.message_id == event.id)
                    .map(|(_, v)| v);

                if let Some(answer) = ans {
                    if let Some(version) = parse_touhou_version(&con) {
                        let old_reaction = touhou_emoji(&answer.answer);
                        let reaction = touhou_emoji(&version);
                        answer.answer = version;
                        channel_id
                            .delete_reaction(&ctx, event.id, None, old_reaction)
                            .ok();
                        channel_id.create_reaction(&ctx, event.id, reaction).ok();
                    }
                }
            }
        }

        Event::MessageDelete(event) => {
            let channel_id = event.channel_id;

            if let Some(chan) = TMQ_COLLECTOR.lock().get_mut(&channel_id) {
                chan.iter()
                    .find(|(_, v)| v.message_id == event.message_id)
                    .map(|v| v.0)
                    .copied()
                    .and_then(|k| chan.remove(&k));
            }
        }

        Event::MessageDeleteBulk(event) => {
            let channel_id = event.channel_id;

            if let Some(chan) = TMQ_COLLECTOR.lock().get_mut(&channel_id) {
                let authors = chan
                    .iter()
                    .filter(|(_, v)| event.ids.contains(&v.message_id))
                    .map(|k| k.0)
                    .copied()
                    .collect::<Vec<UserId>>();

                for author in authors {
                    chan.remove(&author);
                }
            }
        }

        _ => {}
    }
}
