use serde::{Serialize, Deserialize};
use crate::commands::prelude::*;
use magic::traits::MagicIter;
use smallstr::SmallString;
use futures::future::TryFutureExt;
use futures::stream::StreamExt;
use tokio::time::timeout;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use serenity::builder::CreateEmbed;
use serenity::model::channel::ReactionType;
use crate::Result;
use db::{OwnedValue, Value};

const API: &str = "https://api.covid19api.com/summary";
const THUMBNAIL: &str = "https://upload.wikimedia.org/wikipedia/commons/thumb/b/b4/Topeka-leaderboard.svg/200px-Topeka-leaderboard.svg.png";
const WAIT_TIME: Duration = Duration::from_secs(30);
const CACHE_TIME: u64 = 5 * 60;
const DB_KEY: &str = "corona";
const DB_TIME: &str = "c-time";

const REACTIONS: &[&str] = &["◀️", "▶️", "❌"];

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
/// `tomo>leaderboard 5` < this will only show 5 countries per page
async fn leaderboard(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = get_corona_data(&ctx).await?;
    
    let total_c = data.global.total_confirmed;
    let total_r = data.global.total_recovered;
    let total_d = data.global.total_deaths;
        
    let rate_re = get_rate(total_c as f32, total_r as f32);
    let rate_de = get_rate(total_c as f32, total_d as f32);
    
    let info = format!(
        "**Total:** {}\n**Recovered:** {} ({}%)\n**Deaths:** {} ({}%)", 
        total_c, 
        total_r, rate_re, 
        total_d, rate_de,
    );
    
    let per_page: u16 = args
        .single::<u16>()
        .ok()
        .filter(|&v| v > 0 && v <= 50)
        .unwrap_or(10);
    let mut current_page: u16 = 0;
    
    let mut mess = msg.channel_id.send_message(&ctx, |message| {
        message.reactions(REACTIONS.into_iter().map(|&v| v));
        message.embed(|embed| {
            embed.title("Corona Leaderboard");
            embed.timestamp(now());
            embed.color(0x8b0000);
            embed.thumbnail(THUMBNAIL);
            embed.footer(|f| f.text("C = Confirmed | R = Recovered | D = Deaths"));
           
            embed.description(&info);
            
            let show_data = data
                .countries
                .iter()
                .zip(1..)
                .skip((current_page * per_page) as usize)
                .take(per_page as usize);
            
            append_data(show_data, embed)
        });
        
        message
    }).await?;
    
    let mut collector = mess
        .await_reactions(&ctx)
        .author_id(msg.author.id)
        .filter(|reaction| {
            matches!(&reaction.emoji, ReactionType::Unicode(s) if REACTIONS.contains(&s.as_str()))
        })
        .await;
        
    while let Ok(Some(reaction)) = timeout(WAIT_TIME, collector.next()).await {
        let reaction = reaction.as_inner_ref();
        
        let http = Arc::clone(&ctx.http);
        let react = reaction.to_owned();
        tokio::spawn(async move {
            react.delete(http).await.ok();
        });
        
        let emoji = match &reaction.emoji {
            ReactionType::Unicode(s) => s,
            _ => continue,
        };
        
        match emoji.as_str() {
            "◀️" => {
                if current_page == 0 {
                    continue;
                }
                
                current_page -= 1;
            }
            
            "▶️" => {
                if data.countries.len() as u16 - (current_page * per_page) <= per_page {
                    continue;
                }
                
                current_page += 1;
            }
            
            "❌" => break,
            _ => continue
        }
        
        let show_data = data
            .countries
            .iter()
            .zip(1..)
            .skip((current_page * per_page) as usize)
            .take(per_page as usize);
        
        mess.edit(&ctx, |m| m.embed(|embed| {
            embed.title("Corona Leaderboard");
            embed.timestamp(now());
            embed.color(0x8b0000);
            embed.thumbnail(THUMBNAIL);
            embed.footer(|f| f.text("C = Confirmed | R = Recovered | D = Deaths"));
            embed.description(&info);
            
            append_data(show_data, embed)
        })).await?;
    }
    
    drop(collector);
    
    let futs = REACTIONS
        .into_iter()
        .map(|&s| msg.channel_id.delete_reaction(&ctx, mess.id.0, None, s));
        
    futures::future::join_all(futs).await;
    Ok(())
}

fn append_data<'a, 'b, I: IntoIterator<Item=(&'a Country, usize)>>(
    iter: I,
    embed: &'b mut CreateEmbed    
) -> &'b mut CreateEmbed {
    let mut iter = iter.into_iter();
    embed.0.remove("fields");
    
    loop {
        let mut ranking = None;
        let mut many = 0;
        
        let data = iter
            .by_ref()
            .take(10)
            .map(|(v, i)| {
                if ranking.is_none() {
                    ranking = Some(i);
                }
                
                many += 1;
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
            .join('\n');
        
        match ranking {
            Some(r) => {
                let name = format!("#{} - #{}", r, r + many - 1);
                embed.field(name, data, false);
            }
            
            None => break,
        }
    }
    
    embed
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
                .rev()
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
