use crate::commands::prelude::*;
use crate::traits::{CreateEmbed, Embedable, Paginator};
use magic::traits::MagicIter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OsuMatch {
    #[serde(rename = "match")]
    pub osu_match: Match,
    pub games: Vec<Game>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    pub game_id: String,
    pub start_time: String,
    pub end_time: String,
    pub beatmap_id: String,
    pub play_mode: String,
    pub match_type: String,
    pub scoring_type: String,
    pub team_type: String,
    pub mods: String,
    pub scores: Vec<Score>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Score {
    pub slot: String,
    pub team: String,
    pub user_id: String,
    pub score: String,
    pub maxcombo: String,
    pub rank: String,
    pub count50: String,
    pub count100: String,
    pub count300: String,
    pub countmiss: String,
    pub countgeki: String,
    pub countkatu: String,
    pub perfect: String,
    pub pass: String,
    pub enabled_mods: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Match {
    pub match_id: String,
    pub name: String,
    pub start_time: String,
    pub end_time: Option<String>,
}

impl Embedable for OsuMatch {
    fn append(&self, embed: &mut CreateEmbed) {
        let description = format!(
            "ID: {}\nName: {}\nStart at: {}\nUntil: {}\nGame played: {}",
            self.osu_match.match_id,
            self.osu_match.name,
            self.osu_match.start_time,
            self.osu_match
                .end_time
                .to_owned()
                .unwrap_or_else(|| String::from("now")),
            self.games.len(),
        );

        let a: String = self
            .games
            .iter()
            .flat_map(|v| v.scores.iter().map(|v| v.user_id.to_owned()))
            .collect::<std::collections::HashSet<String>>()
            .iter()
            .join(" ");

        embed.description(description);
        embed.field("Players", a, false);
    }
}

impl Paginator for OsuMatch {
    fn append_page(&self, page: core::num::NonZeroUsize, embed: &mut CreateEmbed) {}

    fn total_pages(&self) -> Option<usize> {
        Some(self.games.len())
    }
}

#[command("match")]
async fn osu_match(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let api_key = match crate::read_config().await.apikeys.osu.as_ref() {
        Some(k) => k.to_owned(),
        None => return Ok(()),
    };

    let match_id = args.find::<u64>()?;

    let url = format!(
        "https://osu.ppy.sh/p/api/get_match?k={}&mp={}",
        api_key, match_id
    );
    let req = get_data::<ReqwestClient>(ctx)
        .await
        .ok_or(magic::Error)?
        .get(&url)
        .send()
        .await?
        .json::<OsuMatch>()
        .await?;

    req.send_embed(ctx, msg.channel_id).await?;

    Ok(())
}
