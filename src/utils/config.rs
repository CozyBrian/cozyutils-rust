use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CozyConfig {
    pub gemini_api_key: Option<String>,
    pub backend: Option<String>,
}

fn load_config() -> Option<CozyConfig> {
    read_config().ok()
}

fn resolve_home_dir() -> Option<PathBuf> {
    if let Ok(home) = env::var("HOME") {
        if !home.is_empty() {
            return Some(PathBuf::from(home));
        }
    }

    if let Ok(home) = env::var("USERPROFILE") {
        if !home.is_empty() {
            return Some(PathBuf::from(home));
        }
    }

    None
}

pub fn config_path() -> Option<PathBuf> {
    resolve_home_dir().map(|home| home.join(".cozyutils").join("config.json"))
}

pub fn read_config() -> Result<CozyConfig, String> {
    let path = config_path().ok_or_else(|| "Failed to resolve home directory.".to_string())?;
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read config {}: {}", path.display(), error))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("Failed to parse config {}: {}", path.display(), error))
}

pub fn load_config_or_default() -> CozyConfig {
    load_config().unwrap_or_default()
}

pub fn load_config_api_key() -> Option<String> {
    let config = load_config()?;
    config
        .gemini_api_key
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn load_default_backend() -> Option<String> {
    let config = load_config()?;
    config
        .backend
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn load_gemini_api_key() -> Result<String, String> {
    env::var("GEMINI_API_KEY")
        .ok()
        .or_else(load_config_api_key)
        .ok_or_else(|| {
            "Missing GEMINI_API_KEY environment variable or ~/.cozyutils/config.json entry."
                .to_string()
        })
}

pub fn update_config(
    api_key: Option<Option<&str>>,
    backend: Option<Option<&str>>,
) -> Result<PathBuf, String> {
    let path = config_path().ok_or_else(|| "Failed to resolve home directory.".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create config directory: {}", error))?;
    }

    let existing = load_config_or_default();
    let config = CozyConfig {
        gemini_api_key: match api_key {
            Some(Some(value)) => Some(value.to_string()),
            Some(None) => None,
            None => existing.gemini_api_key,
        },
        backend: match backend {
            Some(Some(value)) => Some(value.to_string()),
            Some(None) => None,
            None => existing.backend,
        },
    };
    let content = serde_json::to_string_pretty(&config)
        .map_err(|error| format!("Failed to serialize config: {}", error))?;
    fs::write(&path, format!("{}\n", content))
        .map_err(|error| format!("Failed to write config: {}", error))?;
    Ok(path)
}

pub fn write_config(api_key: Option<&str>, backend: Option<&str>) -> Result<PathBuf, String> {
    update_config(Some(api_key), Some(backend))
}
