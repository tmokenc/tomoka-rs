use crate::types::GuildConfig;
use crate::Result;
use dashmap::DashMap;
use lib_config::{Config as LibConfig, Environment};
use serde::{Deserialize, Serialize};
use serenity::model::id::GuildId;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub discord_token: String,
    pub prefix: String,
    pub db_path: PathBuf,
    #[serde(default)]
    pub guilds: DashMap<GuildId, GuildConfig>,
    pub temp_dir: Option<PathBuf>,
    pub rgb_evidence: Option<PathBuf>,
    pub radio_stations: Option<PathBuf>,
    pub tmq_source: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let mut config = LibConfig::new();

        config.set_default("prefix", "tomo>")?;
        config.set_default("db_path", "./tomodb")?;
        config.merge(Environment::new())?;

        let res = config.try_into()?;
        Ok(res)
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

        Ok(path)
    }
}
