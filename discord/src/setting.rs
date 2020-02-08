use crate::Result;
use config::{Config, Environment};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct Setting {
    pub discord_token: String,
    pub prefix: String,
    pub db_path: PathBuf,
    pub temp_dir: Option<PathBuf>,
    pub rgb_evidence: Option<PathBuf>,
    pub radio_stations: Option<PathBuf>,
    pub tmq_source: Option<String>,
}

impl Setting {
    pub fn from_env() -> Result<Self> {
        let mut config = Config::new();

        config.set_default("prefix", "tomo>")?;
        config.set_default("db_path", "./tomodb")?;
        config.merge(Environment::new())?;

        let res = config.try_into()?;
        Ok(res)
    }
}
