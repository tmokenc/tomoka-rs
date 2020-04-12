use serde::{Serialize, Deserialize};
use crate::commands::prelude::*;
use magic::traits::MagicIter;
use smallstr::SmallString;
use futures::future::TryFutureExt;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use crate::Result;
use db::{OwnedValue, Value};

const API: &str = "https://api.covid19api.com/summary";
const THUMBNAIL: &str = "https://upload.wikimedia.org/wikipedia/commons/thumb/b/b4/Topeka-leaderboard.svg/200px-Topeka-leaderboard.svg.png";
const CACHE_TIME: u64 = 5 * 60;
const DB_KEY: &str = "corona";
const DB_TIME: &str = "c-time";

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CoronaSummary {
    global: Global,
    countries: Vec<Country>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Global {
    // new_confirmed: i64,
    total_confirmed: u64,
    // new_deaths: i64,
    total_deaths: u64,
    // new_recovered: i64,
    total_recovered: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Country {
    country: String,
    country_code: SmallString<[u8; 2]>,
    // slug: String,
    // new_confirmed: i64,
    total_confirmed: u64,
    // new_deaths: i64,
    total_deaths: u64,
    // new_recovered: i64,
    total_recovered: u64,
    // date: String,
}

#[command]
#[aliases("corona", "top")]
/// Get corona leaderboard
/// Add limit number to limit the result
/// `tomo>leaderboard 10` < this will only show first 10 countries
async fn leaderboard(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let res = get_corona_data(&ctx).await?;
    
    let total_c = res.global.total_confirmed;
    let total_r = res.global.total_recovered;
    let total_d = res.global.total_deaths;
        
    let rate_re = get_rate(total_c as f32, total_r as f32);
    let rate_de = get_rate(total_c as f32, total_d as f32);
    
    let take = args.single::<usize>().unwrap_or(50);
    
    let mut iter = res
        .countries
        .iter()
        .rev()
        .filter(|v| v.total_confirmed != 0)
        .zip(1..)
        .take(take);
    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Corona Leaderboard");
        embed.timestamp(now());
        embed.color(0x8b0000);
        embed.thumbnail(THUMBNAIL);
        embed.footer(|f| f.text("C = Confirmed | R = Recovered | D = Deaths"));
        
        let info = format!(
            "**Total:** {}\n**Recovered:** {} ({}%)\n**Deaths:** {} ({}%)", 
            total_c, 
            total_r, rate_re, 
            total_d, rate_de,
        );
        
        embed.description(info);
        embed.field("TOP 10", data(iter.by_ref().take(10)), false);
        
        let mut top = 20;
        
        for _ in 0..8 {
            let d = data(iter.by_ref().take(10));
            
            if !d.is_empty() {
                let title = format!("#{} ~ #{}", top - 9, top);
                embed.field(title, d, false);
                top += 10;
            } else {
                break
            }
        }
        
        embed
    })).await?;
    
    Ok(())
}

async fn get_corona_data(ctx: &Context) -> Result<CoronaSummary> {
    let db = get_data::<DatabaseKey>(&ctx).await.unwrap();
    let db_ref = Arc::clone(&db);
    
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let last = tokio::task::spawn_blocking(move || {
        match db_ref.get(DB_TIME) {
            Ok(OwnedValue::U64(s)) => s,
            _ => 0,
        }
    }).await?;
    
    if now - last > CACHE_TIME {
        let res = get_data::<ReqwestClient>(ctx).await
            .unwrap()
            .get(API)
            .send()
            .and_then(|v| v.json::<CoronaSummary>())
            .await;
            
        if let Ok(mut data) = res {
            data.countries.sort_by_key(|v| v.total_confirmed);
            data.countries = data
                .countries
                .into_iter()
                .filter(|v| v.total_confirmed != 0)
                .collect();
                
            tokio::task::block_in_place(|| {
                let put = db
                    .put_json(DB_KEY, &data)
                    .and_then(|_| db.put(DB_TIME, &Value::U64(now)));
                    
                if let Err(why) = put {
                    error!("Failed to put the corona data to db\n{:#?}", why);
                }
            });
            
            return Ok(data);
        }
    }
    
    let json = tokio::task::spawn_blocking(move || {
        db.get_json(DB_KEY)
    }).await??;
    
    Ok(json)
}

fn data<'a, I: IntoIterator<Item=(&'a Country, usize)>>(iter: I) -> String {
    iter.into_iter()
        .map(|(v, i)| {
            let (rate_re, rate_de) = rate(&v);
            format!(
                "**{}. {} {}**: C: `{}` | R: `{}` ({}%) | D: `{}` ({}%)", 
                i, 
                code_to_emoji(&v.country_code),
                v.country, 
                v.total_confirmed, 
                v.total_recovered, rate_re,
                v.total_deaths, rate_de,
            )
        })
        .join('\n')
}

fn code_to_emoji(s: &str) -> String {
    match s {
        "OT" => String::from(":cruise_ship:"),
        n => format!(":flag_{}:", n.to_lowercase())
    }
}

fn rate(d: &Country) -> (f32, f32) {
    let confirmed = d.total_confirmed as f32;
    let recovered = d.total_recovered as f32;
    let deaths = d.total_deaths as f32;
    
    let re = get_rate(confirmed, recovered);
    let de = get_rate(confirmed, deaths);
    
    (re, de)
}

fn get_rate(t: f32, a: f32) -> f32 {
    (((a as f32) / (t as f32)) * 1000.0).trunc() / 10.0
}
