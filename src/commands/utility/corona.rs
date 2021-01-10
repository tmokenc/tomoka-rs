use crate::commands::prelude::*;
use crate::traits::Paginator;
use crate::Result;
use futures::future::TryFutureExt;
use magic::traits::MagicIter;
use serde::{Deserialize, Serialize};
use serenity::builder::CreateEmbed;
use smallstr::SmallString;
use std::time::{SystemTime, UNIX_EPOCH};

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

struct CoronaPagination {
    countries: Vec<Country>,
    summary: String,
    per_page: usize,
}

impl CoronaPagination {
    fn new(data: CoronaSummary, per_page: usize) -> Self {
        let total_c = data.global.total_confirmed;
        let total_r = data.global.total_recovered;
        let total_d = data.global.total_deaths;
    
        let rate_re = get_rate(total_c as f32, total_r as f32);
        let rate_de = get_rate(total_c as f32, total_d as f32);
    
        let summary = format!(
            "**Total:** {}\n**Recovered:** {} ({}%)\n**Deaths:** {} ({}%)",
            total_c, total_r, rate_re, total_d, rate_de,
        );
        
        Self {
            countries: data.countries,
            summary,
            per_page,
        }
    }
}

impl Paginator for CoronaPagination {
    fn append_page(&self, page: core::num::NonZeroUsize, embed: &mut CreateEmbed) {
        embed.title("Corona Leaderboard");
        embed.timestamp(now());
        embed.color(0x8b0000);
        embed.thumbnail(THUMBNAIL);
        embed.footer(|f| f.text("C = Confirmed | R = Recovered | D = Deaths"));
        embed.description(&self.summary);
        
        let page = page.get() - 1;
        let mut show_data = self
            .countries
            .iter()
            .zip(1..)
            .skip(page as usize * self.per_page)
            .take(self.per_page);
    
        loop {
            let mut ranking = None;
            let mut many = 0;
    
            let data = show_data
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
                        v.total_recovered,
                        rate_re,
                        v.total_deaths,
                        rate_de,
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
    }

    fn total_pages(&self) -> Option<usize> {
        Some((self.countries.len() / self.per_page) + 1)
    }
}

#[command]
#[aliases("leaderboard", "top")]
/// Get corona leaderboard
/// Add limit number to limit the result
/// `tomo>leaderboard 5` < this will only show 5 countries per page
async fn corona(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = get_corona_data(&ctx).await?;
    let per_page = args
        .single::<usize>()
        .ok()
        .filter(|&v| v > 0 && v <= 50)
        .unwrap_or(10);
        
    CoronaPagination::new(data, per_page).pagination(ctx, msg).await?;
    Ok(())
}

async fn get_corona_data(ctx: &Context) -> Result<CoronaSummary> {
    let db = get_data::<DatabaseKey>(&ctx).await.unwrap();
    let db_ref = db.clone();

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let last: u64 = tokio::task::spawn_blocking(move || db_ref.get(&DB_TIME))
        .await??
        .unwrap_or(0);

    if now - last > CACHE_TIME {
        let res = get_data::<ReqwestClient>(ctx)
            .await
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
                    .insert(&DB_KEY, &data)
                    .and_then(|_| db.insert(&DB_TIME, &now));

                if let Err(why) = put {
                    error!("Failed to put the corona data to db\n{:#?}", why);
                }
            });

            return Ok(data);
        }
    }

    tokio::task::spawn_blocking(move || db.get(&DB_KEY))
        .await??
        .ok_or_else(|| Box::new(magic::Error) as Box<_>)
}

fn code_to_emoji(s: &str) -> String {
    format!(":flag_{}:", s.to_lowercase())
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
