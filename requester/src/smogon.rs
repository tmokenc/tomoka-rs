use crate::Reqwest;
use crate::Result;
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::collections::HashMap;

type DumpPokemonResult = Result<SmogonDumpPokemon>;
type StrategyResult = Result<Vec<SmogonStrategy>>;

const BASICS_ENDPOINT: &str = "https://www.smogon.com/dex/_rpc/dump-basics";
const STRATEGY_ENDPOINT: &str = "https://www.smogon.com/dex/_rpc/dump-pokemon";
const ABILITY_ENDPOINT: &str = "https://www.smogon.com/dex/_rpc/dump-ability";
const MOVE_ENDPOINT: &str = "https://www.smogon.com/dex/_rpc/dump-move";

#[derive(Debug, Deserialize, Serialize)]
pub struct DumpBasics {
    pub pokemon: Vec<Pokemon>,
    pub formats: Vec<Format>,
    pub natures: Vec<Nature>,
    pub abilities: Vec<Ability>,
    // pub moveflags: Vec<_>,
    pub moves: Vec<Move>,
    pub types: Vec<Type>,
    pub items: Vec<Item>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Pokemon {
    
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Format {
    
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Nature {
    
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Ability {
    
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Move {
    pub name: String,
    #[serde(rename = "isNonstandard")]
    pub is_nonstandard: String,
    pub category: String,
    pub power: u8,
    pub accuracy: u8,
    pub priority: u8,
    pub pp: u8,
    pub description: String,
    #[serde(rename = "type")]
    pub kind: String,
    // pub flags: Vec<_>,
    pub genfamily: Vec<Generation>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Type {
    pub name: String,
    pub description: String,
    pub genfamily: Vec<Generation>,
    pub atk_effectives: [(String, f32); 18],
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    pub name: String,
    pub description: String,
    #[serde(rename = "isNonstandard")]
    pub is_nonestandard: String,
    pub genfamily: Vec<Generation>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Generation {
    
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SmogonDumpPokemon {
    pub languages: Vec<String>,
    pub learnset: Vec<String>,
    pub strategies: Vec<SmogonStrategy>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SmogonStrategy {
    pub format: String,
    pub overview: String,
    pub comments: String,
    pub movesets: Vec<MoveSet>,
    pub credits: Option<Credit>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SmogonCommon {
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Credit {
    pub teams: Vec<TeamInfo>,
    #[serde(alias = "writtenBy")]
    pub written_by: Vec<MemberInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeamInfo {
    pub name: String,
    pub members: Vec<MemberInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MemberInfo {
    pub user_id: u32,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
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
    async fn dump_basics(&self, version: &str) -> Result<DumpBasics>;

    async fn dump_pokemon<'a, V: Into<Option<&'a str>> + Send + 'async_trait>(
        &self,
        pokemon: &str,
        version: V,
    ) -> DumpPokemonResult;
    
    /// Simple route for ability and move
    async fn dump_common<'a, V: Into<Option<&'a str>> + Send + 'async_trait>(&self, url: &str, data: &str, version: V) -> Result<SmogonCommon>;
    
    async fn dump_ability<'a, V: Into<Option<&'a str>> + Send + 'async_trait>(&self, data: &str, version: V) -> Result<SmogonCommon> {
        self.dump_common(ABILITY_ENDPOINT, data, version).await       
    }
    
    async fn dump_move<'a, V: Into<Option<&'a str>> + Send + 'async_trait>(&self, data: &str, version: V) -> Result<SmogonCommon> {
        self.dump_common(MOVE_ENDPOINT, data, version).await
    }
    
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
    async fn dump_basics(&self, version: &str) -> Result<DumpBasics> {
        let params = json!({
            "gen": version
        });

        let res = self.post(BASICS_ENDPOINT).json(&params).send().await?.json().await?;
        Ok(res)
    }

    async fn dump_pokemon<'a, V: Into<Option<&'a str>> + Send + 'async_trait>(
        &self,
        pokemon: &str,
        version: V,
    ) -> DumpPokemonResult {
        let version = version.into().unwrap_or("ss");
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
    
    async fn dump_common<'a, V: Into<Option<&'a str>> + Send + 'async_trait>(&self, url: &str, data: &str, version: V) -> Result<SmogonCommon> {
        let version = version.into().unwrap_or("ss");
        let params = json!({
            "gen": version,
            "alias": data
        });
    
        let res = self.post(url).json(&params).send().await?.json().await?;
        Ok(res)
    }
}
