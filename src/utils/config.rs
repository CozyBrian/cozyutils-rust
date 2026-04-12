use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct CozyConfig {
    gemini_api_key: Option<String>,
    backend: Option<String>,
}

fn load_config() -> Option<CozyConfig> {
    let path = config_path()?;
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
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

fn config_path() -> Option<PathBuf> {
    resolve_home_dir().map(|home| home.join(".cozyutils").join("config.json"))
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

pub fn write_config(api_key: Option<&str>, backend: Option<&str>) -> Result<PathBuf, String> {
    let path = config_path().ok_or_else(|| "Failed to resolve home directory.".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create config directory: {}", error))?;
    }

    let existing = load_config();
    let config = CozyConfig {
        gemini_api_key: api_key.map(|value| value.to_string()).or_else(|| {
            existing
                .as_ref()
                .and_then(|config| config.gemini_api_key.clone())
        }),
        backend: backend
            .map(|value| value.to_string())
            .or_else(|| existing.and_then(|config| config.backend)),
    };
    let content = serde_json::to_string_pretty(&config)
        .map_err(|error| format!("Failed to serialize config: {}", error))?;
    fs::write(&path, format!("{}\n", content))
        .map_err(|error| format!("Failed to write config: {}", error))?;
    Ok(path)
}
