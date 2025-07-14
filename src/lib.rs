use std::{env, fmt::Display, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    version: String,
    mod_names: Vec<String>,
    hash_names: Vec<String>,
}

impl Config {
    pub fn new() -> Self {
        Config::default()
    }

    pub fn default_path() -> PathBuf {
        if let Some(path) = env::var_os("PHASH-CONFIG-PATH") {
            return PathBuf::from(path);
        }

        if let Some(home) = env::var_os("HOME") {
            let exe_path = env::current_exe().expect("Could not get exe name");
            let exe_name = exe_path
                .file_name()
                .and_then(|os_str| os_str.to_str())
                .unwrap_or("p-hash-benchmark");

            return PathBuf::from(home).join(".config").join(exe_name);
        }

        env::current_dir()
            .expect("Failed to get current directory")
            .join("config.toml")
    }

    pub fn create_default(path: &PathBuf) -> Result<(), ConfigError> {
        if path.exists() {
            return Err(ConfigError::ConfigExists(format!(
                "File {:?} already exists",
                path
            )));
        }
        let config = Config::default();
        let toml_str =
            toml::to_string(&config).map_err(|e| ConfigError::ConfigError(e.to_string()))?;

        fs::write(path, toml_str).map_err(|e| ConfigError::ConfigError(e.to_string()))?;

        Ok(())
    }

    pub fn load(path: PathBuf) -> Result<Self, ConfigError> {
        let toml_str =
            fs::read_to_string(path).map_err(|e| ConfigError::ConfigError(e.to_string()))?;

        let toml_data: Config =
            toml::from_str(&toml_str).map_err(|e| ConfigError::ConfigError(e.to_string()))?;

        Ok(toml_data)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            mod_names: Vec::new(),
            hash_names: Vec::new(),
        }
    }
}

pub enum ConfigError {
    ConfigError(String),
    ConfigExists(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            ConfigError::ConfigExists(msg) => {
                write!(f, "Config error, config already exists: {}", msg)
            }
        }
    }
}
