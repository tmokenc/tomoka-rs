use crate::commands::prelude::*;
use crate::storages::RawEventList;
use crate::Result;
use crate::traits::ChannelExt;
use colorful::Colorful;
use lazy_static::lazy_static;
use log::info;
use mp3_duration;
use rand::prelude::*;
use regex::Regex;
use serenity::model::event::Event;
use serenity::model::id::{ChannelId, EmojiId, MessageId, UserId};
use serenity::model::misc::EmojiIdentifier;
// use serenity::voice::{ffmpeg_optioned, AudioSource, Bitrate};
use songbird::Bitrate;
use std::collections::HashMap;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::Mutex;

type TmqCollect = Mutex<HashMap<ChannelId, HashMap<UserId, TmqCollector>>>;

lazy_static! {
    static ref TOUHOU_VERSION: HashMap<String, u64> = {
        futures::executor::block_on(async {
            use std::fs::File;
            let config = crate::read_config().await;
            let file = File::open(&config.tmq.as_ref().unwrap().emoji).unwrap();
            drop(config);
            let reader = BufReader::new(file);

            serde_json::from_reader(reader).unwrap()
        })
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
/// The unfinished TouhouMusicQuiz game
/// Try to guess which touhou version that contains the song currently playing
async fn touhou_music_quiz(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    if let Some(channel) = is_playing(&ctx, guild_id).await {
        let to_say = format!("I'm current playing on channel <#{}>", channel.0);
        msg.channel_id.say(&ctx, to_say).await?;
        return Ok(());
    }

    let voice_channel = match get_user_voice_channel(&ctx, guild_id, msg.author.id).await {
        Some(c) => c,
        None => return Ok(()),
    };

    let (handle_lock, _) = match songbird::get(&ctx).await {
        Some(m) => m.join(guild_id, voice_channel).await,
        None => return Ok(()),
    };
    
    handle_lock.lock().await.set_bitrate(Bitrate::BitsPerSecond(192000));

    let duration = {
        let config = crate::read_config().await;
        config.tmq.as_ref().unwrap().duration
    };

    {
        let mut collector = TMQ_COLLECTOR.lock().await;
        if collector.is_empty() {
            let custom_events = get_data::<RawEventList>(&ctx).await.unwrap();

            custom_events.add("tmq", TmqEventHandler).await;
        }

        collector.insert(msg.channel_id, HashMap::new());
    }

    let leave_message = loop {
        let (path, version) = match get_quiz().await {
            Ok(v) => v,
            Err(err) => break err.to_string(),
        };
        
        let audio = match get_audio(path, duration).await {
            Ok(v) => v,
            Err(err) => break err.to_string(),
        };

        info!("The answer is touhou {}", version.as_str().blue());
        handle_lock.lock().await.play_only_source(audio);

        let wair_for = Duration::from_secs_f32(duration - 2.0);
        tokio::time::sleep(wair_for).await;

        let (winners_list, loosers_list): (Vec<_>, Vec<_>) = TMQ_COLLECTOR
            .lock()
            .await
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

        let mut send_embed = msg.channel_id
            .send_embed(&ctx)
            .with_description(response)
            .with_current_timestamp();
            
        if !winners.is_empty() {
            send_embed.field("Winners", winners, true);
        }
        
        send_embed.await?;

        if is_dead_channel(&ctx, voice_channel).await {
            break format!(
                "Leaving channel <#{}>, since there are no human left",
                voice_channel
            );
        }
    };

    handle_lock.lock().await.leave().await?;

    {
        let mut collector = TMQ_COLLECTOR.lock().await;
        collector.remove(&msg.channel_id);

        if collector.is_empty() {
            if let Some(events) = get_data::<RawEventList>(&ctx).await {
                events.remove("tmq").await;
            }
        }
    }

    msg.channel_id.say(&ctx, leave_message).await?;
    Ok(())
}

async fn get_audio(path: impl AsRef<Path>, duration: f32) -> Result<songbird::input::Input> {
    use std::io::{Error, ErrorKind};
    let path = path.as_ref();
    let path_move = path.to_owned();
    let total_time = tokio::task::spawn_blocking(move || {
        mp3_duration::from_path(path_move)
            .map(|v| v.as_secs_f32())
            .map_err(|_| Error::new(ErrorKind::Other, "quickfix"))
    })
    .await??;

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

    songbird::input::ffmpeg_optioned(path, &[], opt)
        .await
        .map_err(|e| format!("{:#?}", e).into())
}

async fn get_quiz() -> Result<(PathBuf, String)> {
    use std::io::{Error, ErrorKind};
    use futures::stream::StreamExt;
    use core::future;

    let config = crate::read_config().await;
    let tmq_config = config
        .tmq
        .as_ref()
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "tmq config notfound"))?;

    let path = &tmq_config.source;
    let mut rng = StdRng::from_entropy();

    let dir = fs::read_dir(path)
        .await?
        .filter_map(|v| future::ready(v.ok()))
        .map(|v| v.path())
        .filter(|v| future::ready(v.is_dir()))
        .collect::<Vec<_>>()
        .await;

    let touhou = dir
        .choose(&mut rng)
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "Notfound touhou folder"))?;

    drop(config);

    let version = touhou
        .file_name()
        .and_then(|v| v.to_str())
        .and_then(parse_touhou_version)
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "Notfound touhou version"))?;

    let list = fs::read_dir(touhou)
        .await?
        .filter_map(|v| async { 
            v.ok().filter(|file| {
                file.file_name()
                    .to_str()
                    .filter(|v| v.ends_with(".mp3") && !v.contains("Player Score"))
                    .is_some()
            }) 
        })
        .collect::<Vec<_>>()
        .await;

    let file = list
        .choose(&mut rng)
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "Notfound touhou music"))?;

    info!("Choosed song for TMQ: {:?}", &file);

    Ok((file.path(), version))
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

fn touhou_emoji(version: &str) -> EmojiIdentifier {
    let id = TOUHOU_VERSION.get(version).unwrap();
    let name = format!("th{}", version.replace(".", "_"));

    EmojiIdentifier {
        animated: false,
        id: EmojiId(*id),
        name,
    }
}

pub(crate) struct TmqEventHandler;

#[async_trait::async_trait]
impl crate::traits::RawEventHandlerRef for TmqEventHandler {
    async fn raw_event_ref(&self, ctx: &Context, ev: &Event) {
        if let Err(why) = tmq_event_handler(ctx, ev).await {
            error!("Error with TmQ:\n{:#?}", why);
        }
    }
}

async fn tmq_event_handler(ctx: &Context, ev: &Event) -> Result<()> {
    match ev {
        Event::MessageCreate(event) => {
            let channel_id = event.message.channel_id;
            let author = event.message.author.id;

            if let Some(chan) = TMQ_COLLECTOR.lock().await.get_mut(&channel_id) {
                if chan.contains_key(&author) {
                    return Ok(());
                }

                let touhou_version = parse_touhou_version(&event.message.content);
                if let Some(version) = touhou_version {
                    let emoji = touhou_emoji(&version);
                    let collector = TmqCollector {
                        message_id: event.message.id,
                        answer: version,
                    };

                    chan.insert(author, collector);
                    event.message.react(ctx, emoji).await?;
                }
            }
        }

        Event::MessageUpdate(event) => {
            let channel_id = event.channel_id;

            let content = event.content.as_ref();
            if let (Some(chan), Some(con)) =
                (TMQ_COLLECTOR.lock().await.get_mut(&channel_id), content)
            {
                let ans = chan
                    .iter_mut()
                    .find(|(_, v)| v.message_id == event.id)
                    .map(|(_, v)| v);

                if let Some(answer) = ans {
                    if let Some(version) = parse_touhou_version(&con) {
                        let old_reaction = touhou_emoji(&answer.answer);
                        let reaction = touhou_emoji(&version);
                        answer.answer = version;
                        let deletion =
                            channel_id.delete_reaction(&ctx, event.id, None, old_reaction);
                        let creation = channel_id.create_reaction(&ctx, event.id, reaction);
                        futures::future::try_join(deletion, creation).await?;
                    }
                }
            }
        }

        Event::MessageDelete(event) => {
            let channel_id = event.channel_id;

            if let Some(chan) = TMQ_COLLECTOR.lock().await.get_mut(&channel_id) {
                chan.iter()
                    .find(|(_, v)| v.message_id == event.message_id)
                    .map(|v| v.0)
                    .copied()
                    .and_then(|k| chan.remove(&k));
            }
        }

        Event::MessageDeleteBulk(event) => {
            let channel_id = event.channel_id;

            if let Some(chan) = TMQ_COLLECTOR.lock().await.get_mut(&channel_id) {
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

    Ok(())
}
