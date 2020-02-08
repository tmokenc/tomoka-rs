use serde::{Serialize, Deserialize};

const API_ENDPOINT: &str = "https://pokeapi.co/api/v2/";

#[derive(Debug, Serialize, Deserialize)]
pub struct PokeApiData {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PokemonData {
    pub abilities: Vec<Ability>,
    #[serde(rename = "base_experience")]
    pub base_experience: i64,
    pub forms: Vec<PokeApiData>,
    #[serde(rename = "game_indices")]
    pub game_indices: Vec<Index>,
    pub height: i64,
    #[serde(rename = "held_items")]
    pub held_items: Vec<HeldItem>,
    pub id: i64,
    #[serde(rename = "is_default")]
    pub is_default: bool,
    #[serde(rename = "location_area_encounters")]
    pub location_area_encounters: String,
    pub moves: Vec<Move>,
    pub name: String,
    pub order: i64,
    pub species: PokeApiData,
    pub sprites: Sprites,
    pub stats: Vec<Stat>,
    pub types: Vec<Type>,
    pub weight: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ability {
    pub ability: PokeApiData,
    #[serde(rename = "is_hidden")]
    pub is_hidden: bool,
    pub slot: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    #[serde(rename = "game_index")]
    pub game_index: i64,
    pub version: PokeApiData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeldItem {
    pub item: PokeApiData,
    #[serde(rename = "version_details")]
    pub version_details: Vec<VersionDetail>,
}

pub struct Move {
    #[serde(rename = "move")]
    pub move_field: PokeApiData,
    #[serde(rename = "version_group_details")]
    pub version_group_details: Vec<VersionGroupDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionDetail {
    pub rarity: i64,
    pub version: PokeApiData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionGroupDetail {
    pub level_learned_at: i64,
    pub move_learn_method: PokeApiData,
    pub version_group: PokeApiData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sprites {
    pub back_default: Option<String>,
    pub back_female: Option<String>,
    pub back_shiny: Option<String>,
    pub back_shiny_female: Option<String>,
    pub front_default: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny: Option<String>,
    pub front_shiny_female: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stat {
    pub base_stat: i64,
    pub effort: i64,
    pub stat: PokeApiData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Type {
    pub slot: i64,
    #[serde(rename = "type")]
    pub type_field: PokeApiData,
}

#[async_trait]
pub trait PokeApi {
    async fn pokemon<D: AsRef<str> + Send + 'async_trait>(&self, data: D) -> PokemonData;
    async fn r#type<D: AsRef<str> + Send + 'async_trait>(&self, data: D) -> TypeData;
    async fn r#move<D: AsRef<str> + Send + 'async_trait>(&self, data: D) -> MoveData;
    async fn ability<D: AsRef<str> + Send + 'async_trait>(&self, data: D) -> AbilityData;
}


#[async_trait]
impl PokeApi for crate::Reqwest {
    
}