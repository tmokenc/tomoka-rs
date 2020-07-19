use crate::types::GuildConfig;
use crate::Result;
use dashmap::DashMap;
use db::DbInstance;
use lib_config::{Config as LibConfig, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};
use serenity::model::id::{EmojiId, GuildId, UserId};
use smallstr::SmallString;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;
use tokio::io::AsyncWriteExt as _;

#[derive(Debug, Deserialize, Serialize)]
pub struct Database {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Color {
    pub information: u64,
    pub error: u64,
    pub message_update: u64,
    pub message_delete: u64,
    pub lovely: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Emoji {
    pub sauce: String,
    pub nhentai: String,
    pub sadkaede: String,
    pub pokemon: Option<PokemonTypeEmoji>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PokemonTypeEmoji {
    pub normal: String,
    pub fire: String,
    pub water: String,
    pub grass: String,
    pub poison: String,
    pub dark: String,
    pub ghost: String,
    pub steel: String,
    pub fighting: String,
    pub flying: String,
    pub dragon: String,
    pub rock: String,
    pub ground: String,
    pub psychic: String,
    pub electric: String,
    pub ice: String,
    pub bug: String,
    pub fairy: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ApiKeys {
    pub google: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Rgb {
    pub evidence: PathBuf,
    pub tu: Vec<SmallString<[u8; 8]>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TouhouMusicQuest {
    pub duration: f32,
    pub source: PathBuf,
    pub emoji: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sauce {
    pub wait_duration: u16,
    pub thumbnail: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SadKaede {
    pub cookie: Option<String>,
    pub wait_duration: u16,
    pub thumbnail: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Nhentai {
    pub wait_duration: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Time {
    pub format: SmallString<[u8; 24]>,
    pub zones: Vec<TimeZone>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeZone {
    pub name: Option<SmallString<[u8; 32]>>,
    pub zone: SmallString<[u8; 32]>,
}

impl PokemonTypeEmoji {
    #[inline]
    pub fn get(&self, t: &str) -> Option<&str> {
        match t.to_lowercase().as_str() {
            "fire" => Some(&self.fire),
            "water" => Some(&self.water),
            "grass" => Some(&self.grass),
            "steel" => Some(&self.steel),
            "normal" => Some(&self.normal),
            "bug" => Some(&self.bug),
            "electric" => Some(&self.electric),
            "dragon" => Some(&self.dragon),
            "dark" => Some(&self.dark),
            "fighting" => Some(&self.fighting),
            "psychic" => Some(&self.psychic),
            "poison" => Some(&self.poison),
            "ice" => Some(&self.ice),
            "ghost" => Some(&self.ghost),
            "ground" => Some(&self.ground),
            "rock" => Some(&self.rock),
            "flying" => Some(&self.flying),
            "fairy" => Some(&self.fairy),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub prefix: SmallString<[u8; 8]>,
    pub master_prefix: SmallString<[u8; 8]>,
    pub cmd_blacklist: Vec<SmallString<[u8; 10]>>,
    pub temp_dir: Option<PathBuf>,
    pub eliza_brain: String,
    pub max_cache_file_size: u32,
    pub image_search_depth: u16,
    pub respect_emoji: Option<EmojiId>,
    pub radio_stations: Option<PathBuf>,
    pub disable_auto_cmd: Vec<SmallString<[u8; 14]>>,
    #[serde(default)]
    pub masters: HashSet<UserId>,
    pub rgb: Option<Rgb>,
    pub tmq: Option<TouhouMusicQuest>,
    pub database: Database,
    pub color: Color,
    pub emoji: Emoji,
    pub time: Time,
    pub sauce: Sauce,
    pub sadkaede: SadKaede,
    pub nhentai: Nhentai,
    #[serde(default)]
    pub guilds: DashMap<GuildId, GuildConfig>,
    #[serde(default)]
    pub apikeys: ApiKeys,
}

//pub struct SaveOption {
//    pub path: PathBuf,
// pub file_type: FileType,
//}

// pub enum FileType { Json, Toml }
//
// impl fmt::Display for FileType {
// fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// match self {
// Self::Json => write!(f, "json"),
// Self::Toml => write!(f, "toml"),
// }
// }
// }

impl Config {
    /// Initial the config
    /// This will read config from DefaultConfig > Config file (if exist) > Environment
    /// Data in environment variable will have priority over the config file
    pub fn init() -> Result<Self> {
        let mut config = LibConfig::new();
        let default_config = include_str!("../../assets/data/default_config.toml");

        config.merge(File::from_str(default_config, FileFormat::Toml))?;
        config.merge(File::with_name("./config.toml").required(false))?;
        config.merge(Environment::new())?;

        let res = config.try_into()?;
        Ok(res)
    }

    /// Perform the same action as `init`, but replace self instead of create new
    /// for some reason, keeping the guilds config will make it crash
    /// that's why creating the new one instead and then fetch the guilds config
    /// from the database *again*, because of this, the DbInstance is needed
    /// This *maybe* fix in the future
    pub fn reload(&mut self, db: &DbInstance) -> Result<()> {
        let mut config = LibConfig::new();
        let default_config = include_str!("../../assets/data/default_config.toml");

        config.merge(File::from_str(default_config, FileFormat::Toml))?;
        config.merge(File::with_name("./config.toml"))?;
        config.merge(Environment::new())?;

        *self = config.try_into()?;

        let db_data = db.open("GuildConfig")?.get_all::<GuildId, GuildConfig>();
        let guilds_config = &self.guilds;

        for (k, v) in db_data {
            guilds_config.insert(k, v);
        }

        Ok(())
    }

    /// Save config to file using JSON format
    /// If the provided path is a directory, it will create a `config_{timestamp}.json` inside of that
    /// If the file exists, this will add a `.bak` to that file and do the work.
    /// Return the `PathBuf` of the saved file
    pub async fn save_file<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let mut path = path.as_ref().to_path_buf();

        if path.is_dir() {
            let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
            path.push(format!("config_{}.json", time.as_millis()));
        }

        if fs::metadata(&path).await.is_ok() {
            let mut file_name = path.file_name().unwrap_or_default().to_os_string();

            file_name.push(".bak");

            let mut new_path = path.clone();
            new_path.set_file_name(file_name);

            fs::rename(&path, new_path).await?;
        }

        let data = serde_json::to_vec_pretty(self)?;
        let mut file = fs::File::create(&path).await?;

        file.write_all(data.as_slice()).await?;

        // match path.extension().and_then(|v| v.to_str()) {
        // Some(v) if v == "json" => {
        // serde_json::to_writer_pretty(&mut file, self)?
        // }
        //
        // Some(v) if v == "toml" => {
        //    convert it to json value first and then convert back to toml
        //    this is to avoid the `KeyNotString` error
        // let value = serde_json::to_value(self)?;
        // let data = toml::to_string_pretty(&value)?;
        // file.write_all(data.as_bytes())?
        // }
        // _ => {}
        // }

        Ok(path)
    }
}
