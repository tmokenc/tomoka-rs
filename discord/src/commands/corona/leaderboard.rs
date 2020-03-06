use serde::Deserialize;
use crate::commands::prelude::*;
use magic::traits::MagicIter;

const API: &str = "https://api.coronatracker.com/v2/analytics/country";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CoronaResult {
    // pub country_code: String,
    pub country_name: String,
    // pub lat: f64,
    // pub lng: f64,
    pub confirmed: u64,
    pub deaths: u64,
    pub recovered: u64,
    // pub date_as_of: String,
}

#[command]
fn leaderboard(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let req = get_data::<ReqwestClient>(&ctx).unwrap();
    let res: Vec<CoronaResult> = block_on(async {
        req.get(API).send().await?.json().await
    })?;
    
    let (total_c, total_r, total_d) = res
        .iter()
        .fold((0u64, 0u64, 0u64), |v, x| {
            (v.0 + x.confirmed, v.1 + x.recovered, v.2 + x.deaths)
        });
    
    let take = args.single::<usize>().unwrap_or(res.len());
    let mut iter = res.into_iter().zip(1..).take(take);
    
    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Corona Leaderboard");
        embed.timestamp(now());
        embed.color(0x8b0000);
        embed.footer(|f| f.text("C = Confirmed | R = Recovered | D = Deaths"));
        
        let info = format!("**Total:** {}\n**Recovered:** {}\n**Deaths:** {}", total_c, total_r, total_d);
        
        embed.description(info);
        embed.field("TOP 10", data(iter.by_ref().take(10)), false);
        
        let mut top = 30;
        
        loop {
            let d = data(iter.by_ref().take(20));
            
            if !d.is_empty() {
                let title = format!("#{} ~ #{}", top - 19, top);
                embed.field(title, d, false);
                top += 20;
            } else {
                break embed;
            }
        }
    }))?;
    
    Ok(())
}

fn data<I: IntoIterator<Item=(CoronaResult, usize)>>(iter: I) -> String {
    iter.into_iter()
        .map(|(v, i)| format!("**{}. {}**: C: `{}` | R: `{}` | D: `{}`", i, v.country_name, v.confirmed, v.recovered, v.deaths))
        .join('\n')
}