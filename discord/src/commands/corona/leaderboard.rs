use serde::Deserialize;
use crate::commands::prelude::*;
use magic::traits::MagicIter;

const API: &str = "https://api.coronatracker.com/v2/analytics/country";
const THUMBNAIL: &str = "https://upload.wikimedia.org/wikipedia/commons/thumb/b/b4/Topeka-leaderboard.svg/200px-Topeka-leaderboard.svg.png";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CoronaResult {
    pub country_code: String,
    pub country_name: String,
    // pub lat: f64,
    // pub lng: f64,
    pub confirmed: u64,
    pub deaths: u64,
    pub recovered: u64,
    // pub date_as_of: String,
}

#[command]
#[aliases("corona", "top")]
/// Get corona leaderboard
/// Add limit number to limit the result
/// `tomo>leaderboard 10` < this will only show first 10 countries
async fn leaderboard(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let res: Vec<CoronaResult> = get_data::<ReqwestClient>(&ctx).await
        .unwrap()
        .get(API)
        .send().await?
        .json().await?;
    
    let (total_c, total_r, total_d) = res
        .iter()
        .fold((0u64, 0u64, 0u64), |v, x| {
            (v.0 + x.confirmed, v.1 + x.recovered, v.2 + x.deaths)
        });
        
    let rate_re = get_rate(total_c as f32, total_r as f32);
    let rate_de = get_rate(total_c as f32, total_d as f32);
    
    let take = args.single::<usize>().unwrap_or(res.len());
    let mut iter = res.into_iter().zip(1..).take(take);
    
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

fn data<I: IntoIterator<Item=(CoronaResult, usize)>>(iter: I) -> String {
    iter.into_iter()
        .map(|(v, i)| {
            let (rate_re, rate_de) = rate(&v);
            format!(
                "**{}. {} {}**: C: `{}` | R: `{}` ({}%) | D: `{}` ({}%)", 
                i, 
                code_to_emoji(&v.country_code),
                v.country_name, 
                v.confirmed, 
                v.recovered, rate_re,
                v.deaths, rate_de,
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

fn rate(d: &CoronaResult) -> (f32, f32) {
    let confirmed = d.confirmed as f32;
    let recovered = d.recovered as f32;
    let deaths = d.deaths as f32;
    
    let re = get_rate(confirmed, recovered);
    let de = get_rate(confirmed, deaths);
    
    (re, de)
}

fn get_rate(t: f32, a: f32) -> f32 {
    (((a as f32) / (t as f32)) * 1000.0).trunc() / 10.0
}