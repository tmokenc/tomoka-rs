use crate::Reqwest;
use crate::Result;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

type DumpResult = Result<SmogonDumpPokemon>;
type StrategyResult = Result<Vec<SmogonStrategy>>;

const STRATEGY_ENDPOINT: &str = "https://www.smogon.com/dex/_rpc/dump-pokemon";

#[derive(Deserialize)]
pub struct SmogonDumpPokemon {
    pub languages: Vec<String>,
    pub learnset: Vec<String>,
    pub strategies: Vec<SmogonStrategy>,
}

#[derive(Deserialize)]
pub struct SmogonStrategy {
    pub format: String,
    pub overview: String,
    pub comments: String,
    pub movesets: Vec<MoveSet>,
    pub credits: Option<Credit>,
}

#[derive(Deserialize)]
pub struct Credit {
    pub teams: Vec<TeamInfo>,
    #[serde(alias = "writtenBy")]
    pub written_by: Vec<MemberInfo>,
}

#[derive(Deserialize)]
pub struct TeamInfo {
    pub name: String,
    pub members: Vec<MemberInfo>,
}

#[derive(Deserialize)]
pub struct MemberInfo {
    pub user_id: u32,
    pub username: String,
}

#[derive(Deserialize)]
pub struct MoveSet {
    pub name: String,
    pub pokemon: String,
    pub level: u8,
    pub description: String,
    pub abilities: Vec<String>,
    pub items: Vec<String>,
    pub moveslots: Vec<Vec<String>>,
    pub evconfigs: Vec<StatsConfig>,
    pub ivconfigs: Vec<StatsConfig>,
    pub natures: Vec<String>,
}

type StatsConfig = HashMap<String, u8>;

// #[derive(Deserialize)]
// pub struct StatsConfig {
//     pub hp: u8,
//     pub atk: u8,
//     pub def: u8,
//     pub spa: u8,
//     pub spd: u8,
//     pub spe: u8,
// }

impl MoveSet {
    #[inline]
    pub fn ability(&self) -> String {
        self.abilities.join(" / ")
    }

    #[inline]
    pub fn nature(&self) -> String {
        self.natures.join(" / ")
    }

    pub fn ev_config(&self) -> Vec<String> {
        self.evconfigs
            .iter()
            .map(|stats| {
                stats
                    .iter()
                    .filter(|(_, value)| **value > 0)
                    .map(|(k, v)| stats_display(&k.to_owned(), *v))
                    .collect::<Vec<_>>()
                    .join(" / ")
            })
            .collect()
    }

    pub fn iv_config(&self) -> Vec<String> {
        self.ivconfigs
            .iter()
            .map(|stats| {
                stats
                    .iter()
                    .filter(|(_, value)| **value < 31)
                    .map(|(k, v)| stats_display(&k.to_owned(), *v))
                    .collect::<Vec<_>>()
                    .join(" / ")
            })
            .collect()
    }
}

fn stats_display(stats: &str, value: u8) -> String {
    let k = match stats {
        "hp" => "HP",
        "atk" => "Attack",
        "def" => "Defend",
        "spa" => "Sp. Attack",
        "spd" => "Sp. Defend",
        "spe" => "Speed",
        _ => "Unknown stats",
    };

    format!("{} {}", value, k)
}

#[async_trait]
pub trait SmogonRequester {
    async fn dump_pokemon<'a, V: Into<Option<&'a str>> + Send + 'async_trait>(
        &self,
        pokemon: &str,
        version: V,
    ) -> DumpResult;

    async fn strategy<'a, V: Into<Option<&'a str>> + Send + 'async_trait>(
        &self,
        pokemon: &str,
        version: V,
    ) -> StrategyResult {
        let result = self.dump_pokemon(pokemon, version).await?;
        Ok(result.strategies)
    }
}

#[async_trait]
impl SmogonRequester for Reqwest {
    async fn dump_pokemon<'a, V: Into<Option<&'a str>> + Send + 'async_trait>(
        &self,
        pokemon: &str,
        version: V,
    ) -> DumpResult {
        let version = version.into().unwrap_or("sm");
        let params = json!({
            "gen": version,
            "alias": pokemon,
            "language": "en"
        });

        let res = self
            .post(STRATEGY_ENDPOINT)
            .json(&params)
            .send()
            .await?
            .json()
            .await?;
        Ok(res)
    }
}
