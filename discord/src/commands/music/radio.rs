use crate::commands::prelude::*;
use crate::types::{Music, Playable, PlayingInfo, PlayingSignal};
use crate::Result;
use serde::{Deserialize, Serialize};
use serenity::model::id::UserId;
use serenity::voice::{ffmpeg, AudioSource, Bitrate};
use std::env;
use std::fs;
use std::io::BufReader;
use std::io::{Error as IoError, ErrorKind};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
pub struct Radio {
    pub name: String,
    pub url: String,
    pub scrap_info: Option<RadioScrap>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RadioScrap {
    url: String,
    title: String,
    artist: Option<String>,
    album_art: Option<String>,
    album: Option<String>,
    circle: Option<String>,
    year: Option<String>,
}

impl Playable for Radio {
    fn get_info(&self) -> Result<PlayingInfo> {
        Ok(PlayingInfo {
            title: String::from("Test"),
            ..Default::default()
        })
    }
}

#[command]
fn radio(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
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

    let station = args.rest();
    let station = match find_station(&station) {
        Some(s) => s,
        None => {
            msg.channel_id
                .say(ctx, format!("Cannot find the {} radio station", station))?;
            return Ok(());
        }
    };

    let voice_manager = get_voice_manager(&ctx);
    let mut voice = {
        let mut manager = voice_manager.lock();
        match manager.join(guild_id, voice_channel) {
            Some(c) => c.clone(),
            None => return Ok(()),
        }
    }; // youtubedl -s -e --get-id --get-duration

    let url = station.url.to_owned();
    let stream = get_radio_stream(&url)?;
    let mut music = voice.play_only(stream);

    let (check, signalisation) = Music::new(msg.channel_id, station);

    let manager = ctx.data.read().get::<MusicManager>().cloned().unwrap();
    manager.lock().insert(guild_id, check);

    for signal in signalisation {
        match signal {
            PlayingSignal::Pause => {
                music.lock().pause();
                voice.stop();
            }

            PlayingSignal::Resume => {
                if !music.lock().playing {
                    let stream = get_radio_stream(&url)?;
                    music = voice.play_only(stream);
                }
            }

            PlayingSignal::Stop => break,
            _ => {}
        }
    }

    Ok(())
}

fn get_radio_stream<S: AsRef<str>>(s: S) -> Result<Box<dyn AudioSource>> {
    let audio = ffmpeg(s.as_ref())?;
    Ok(audio)
}

fn parse_radio_file() -> Result<Vec<Radio>> {
    let config = crate::read_config();
    let path = config
        .radio_stations
        .as_ref()
        .ok_or_else(|| IoError::new(ErrorKind::NotFound, "No radio station was set"))?;

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let json: Vec<Radio> = serde_json::from_reader(reader)?;
    Ok(json)
}

#[inline]
fn find_station<S: AsRef<str>>(station: S) -> Option<Radio> {
    parse_radio_file().ok().and_then(|stations| {
        stations
            .into_iter()
            .find(|v| v.name.to_lowercase().starts_with(station.as_ref()))
    })
}
