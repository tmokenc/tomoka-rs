#![allow(clippy::cast_lossless)]

use crate::commands::prelude::*;
use magic::dark_magic::{number_to_rgb, progress_bar};
use std::mem;

pub const HEART_URL: &str = "https://i.imgur.com/YpRPxeS.png";

#[command]
#[usage = "?[@someone] ?[@another one]"]
/// Check love comparative
async fn love(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut person = match msg.mentions.get(0) {
        Some(user) => user,
        None => &msg.author,
    };

    let mut person2 = match msg.mentions.get(1) {
        Some(user) => user,
        None => &msg.author,
    };

    let (point, first) = calculate_love(person.id.0, person2.id.0);
    let point_str = format!("{}%", point);
    if !first {
        mem::swap(&mut person, &mut person2);
    }
    
    let config = crate::read_config().await;
    let color = config.color.lovely;
    drop(config);

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|embed| {
            embed.title("Thước đo tình yêu");
            embed.description(format!(
                ":sparkling_heart: **{}**\n:sparkling_heart: **{}**",
                person.name, person2.name
            ));
            embed.thumbnail(HEART_URL);
            embed.field(&point_str, progress_bar(point, 18), false);
            embed.field(&point_str, get_msg(point), false);
            
            embed.color(color);
            
            embed
        })
    }).await?;

    Ok(())
}

fn calculate_love(id1: u64, id2: u64) -> (u8, bool) {
    let id1 = id1.swap_bytes();
    let id2 = id2.swap_bytes();
    let first = id1 > id2;
    let (a, b) = if first {
        (encode_id(id1), encode_id(id2))
    } else {
        (encode_id(id2), encode_id(id1))
    };

    let mut rate = ((a as f32 / b as f32) * 100.0) as u16;

    while rate > 255 {
        rate /= 2;
    }

    (rate as u8, first)
}

fn encode_id(id: u64) -> u8 {
    let (r, g, b) = number_to_rgb(id);
    (r / 3) + (g / 3) + (b / 3)
}

fn get_msg(point: u8) -> String {
    String::from(match point {
        0..=20 => "Quay đầu là bờ",
        21..=40 => "Có vẻ thú vị",
        41..=60 => "Giới giang hồ hiểm ác khó lường trước được điều gì",
        61..=80 => "Khá hợp nhau đấy, có triển vọng :kissing_heart:",
        81..=94 => "Duyên phận đã định :heart_eyes:",
        95..=99 => "Hai con người của định mệnh đây rồi :heartpulse:",
        _ => ":heartpulse: Hai con người này sinh ra là để dành cho nhau :heartpulse:",
    })
}
