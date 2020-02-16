use crate::types::GuildConfig;
use crate::Result;
use dashmap::DashMap;
use db::DbInstance;
use lib_config::{Config as LibConfig, Environment, File, FileFormat};
use magic::bytes_to_le_u64;
use serde::{Deserialize, Serialize};
use serenity::model::id::{EmojiId, GuildId};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Deserialize, Serialize)]
pub struct Database {
    pub path: String,
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
pub struct Rgb {
    pub evidence: PathBuf,
    pub tu: Vec<String>,
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
    pub emoji: Option<EmojiId>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SadKaede {
    pub cookie: Option<String>,
    pub wait_duration: u16,
    pub emoji: Option<EmojiId>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Time {
    pub format: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub prefix: String,
    pub temp_dir: Option<PathBuf>,
    pub max_cache_file_size: u32,
    pub image_search_depth: u16,
    pub database: Database,
    pub disable_auto_cmd: Vec<String>,
    pub radio_stations: Option<PathBuf>,
    pub color: Color,
    pub rgb: Option<Rgb>,
    pub tmq: Option<TouhouMusicQuest>,
    pub time: Time,
    pub sauce: Sauce,
    pub sadkaede: SadKaede,
    #[serde(default)]
    pub guilds: DashMap<GuildId, GuildConfig>,
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

        let db_data = db.open("GuildConfig")?.get_all_json::<GuildConfig>()?;
        let guilds_config = &self.guilds;

        for (k, v) in db_data {
            let key = bytes_to_le_u64(k).into();
            guilds_config.insert(key, v);
        }

        Ok(())
    }

    /// Save config to file using JSON format
    /// If the provided path is a directory, it will create a `config_{timestamp}.json` inside of that
    /// If the file exists, this will add a `.bak` to that file and do the work.
    /// Return the `PathBuf` of the saved file
    pub fn save_file<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let mut path = path.as_ref().to_path_buf();

        if path.is_dir() {
            let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
            path.push(format!("config_{}.json", time.as_millis()));
        }

        if fs::metadata(&path).is_ok() {
            let mut file_name = path.file_name().unwrap_or_default().to_os_string();

            file_name.push(".bak");

            let mut new_path = path.clone();
            new_path.set_file_name(file_name);

            fs::rename(&path, new_path)?;
        }

        let mut file = fs::File::create(&path)?;
        serde_json::to_writer_pretty(&mut file, self)?;

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
