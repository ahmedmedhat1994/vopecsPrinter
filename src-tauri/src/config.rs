use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub domain_url: String,

    #[serde(default)]
    pub key: String,

    #[serde(default)]
    pub printer_name: Option<String>,

    #[serde(default)]
    pub printer_mappings: HashMap<String, String>,

    #[serde(default)]
    pub open_drawer_after_print: bool,

    #[serde(default)]
    pub drawer_pin: u8,

    #[serde(default)]
    pub polling_interval: u64,

    #[serde(default)]
    pub auto_start: bool,
}

impl Config {
    pub fn new() -> Self {
        Config {
            domain_url: String::new(),
            key: String::new(),
            printer_name: None,
            printer_mappings: HashMap::new(),
            open_drawer_after_print: false,
            drawer_pin: 0,
            polling_interval: 5000,
            auto_start: false,
        }
    }
}

/// Get the config directory path
pub fn get_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".vopecsprinter")
}

/// Get the config file path
pub fn get_config_path() -> PathBuf {
    get_config_dir().join("config.json")
}

/// Ensure the config directory exists
pub fn ensure_config_dir() -> Result<()> {
    let config_dir = get_config_dir();
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
    }
    Ok(())
}

/// Load config from file
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path();

    if !config_path.exists() {
        // Return default config if file doesn't exist
        return Ok(Config::new());
    }

    let content = fs::read_to_string(&config_path)
        .context("Failed to read config file")?;

    let config: Config = serde_json::from_str(&content)
        .context("Failed to parse config file")?;

    Ok(config)
}

/// Save config to file
pub fn save_config(config: &Config) -> Result<()> {
    ensure_config_dir()?;

    let config_path = get_config_path();
    let content = serde_json::to_string_pretty(config)
        .context("Failed to serialize config")?;

    fs::write(&config_path, content)
        .context("Failed to write config file")?;

    Ok(())
}

/// Get mapped printer name
pub fn get_mapped_printer(config: &Config, api_printer_name: &str) -> Option<String> {
    config.printer_mappings.get(api_printer_name).cloned()
}
