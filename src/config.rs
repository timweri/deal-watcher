use std::path::PathBuf;

use serde::Deserialize;

fn default_healthz_port() -> u16 {
    8080
}

/// All configuration lives here, read once at startup from the environment.
/// This is the only place in the program that reads env vars.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub telegram_access_token: String,
    pub telegram_chat_id: String,
    pub data: PathBuf,
    pub time_window: i64,
    #[serde(default = "default_healthz_port")]
    pub healthz_port: u16,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        Ok(envy::from_env::<Config>()?)
    }
}
