use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CozyConfig {
    #[serde(default, skip_serializing_if = "ConfigDefaults::is_empty")]
    pub defaults: ConfigDefaults,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub providers: BTreeMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ConfigDefaults {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub commands: BTreeMap<String, CommandDefaults>,
}

impl ConfigDefaults {
    pub fn is_empty(&self) -> bool {
        self.provider.is_none() && self.commands.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CommandDefaults {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderType {
    Gemini,
    OpenaiCompatible,
    Opencode,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProviderConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<ProviderType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedCommandSettings {
    pub provider_name: String,
    pub provider: ProviderConfig,
    pub model: String,
    pub api_key: Option<String>,
}

fn starter_command_defaults() -> BTreeMap<String, CommandDefaults> {
    let mut commands = BTreeMap::new();
    commands.insert(
        "cmsg".to_string(),
        CommandDefaults {
            provider: Some("deepseek".to_string()),
            model: Some("deepseek-chat".to_string()),
        },
    );
    commands.insert(
        "prmsg".to_string(),
        CommandDefaults {
            provider: Some("openrouter".to_string()),
            model: Some("openai/gpt-5.4-mini".to_string()),
        },
    );
    commands
}

fn resolve_home_dir() -> Option<PathBuf> {
    if let Ok(home) = env::var("HOME")
        && !home.is_empty()
    {
        return Some(PathBuf::from(home));
    }

    if let Ok(home) = env::var("USERPROFILE")
        && !home.is_empty()
    {
        return Some(PathBuf::from(home));
    }

    None
}

fn trim_to_option(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn builtin_provider(
    provider_type: ProviderType,
    base_url: Option<&str>,
    api_key_env: Option<&str>,
    default_model: &str,
) -> ProviderConfig {
    ProviderConfig {
        r#type: Some(provider_type),
        base_url: base_url.map(|value| value.to_string()),
        api_key_env: api_key_env.map(|value| value.to_string()),
        api_key: None,
        default_model: Some(default_model.to_string()),
    }
}

pub fn builtin_providers() -> BTreeMap<String, ProviderConfig> {
    let mut providers = BTreeMap::new();
    providers.insert(
        "deepseek".to_string(),
        builtin_provider(
            ProviderType::OpenaiCompatible,
            Some("https://api.deepseek.com/v1"),
            Some("DEEPSEEK_API_KEY"),
            "deepseek-chat",
        ),
    );
    providers.insert(
        "gemini".to_string(),
        builtin_provider(
            ProviderType::Gemini,
            None,
            Some("GEMINI_API_KEY"),
            "gemini-3-flash-preview",
        ),
    );
    providers.insert(
        "opencode".to_string(),
        builtin_provider(ProviderType::Opencode, None, None, "openai/gpt-5.4-mini"),
    );
    providers.insert(
        "openrouter".to_string(),
        builtin_provider(
            ProviderType::OpenaiCompatible,
            Some("https://openrouter.ai/api/v1"),
            Some("OPENROUTER_API_KEY"),
            "openai/gpt-5.4-mini",
        ),
    );
    providers
}

pub fn starter_config() -> CozyConfig {
    CozyConfig {
        defaults: ConfigDefaults {
            provider: Some("openrouter".to_string()),
            commands: starter_command_defaults(),
        },
        providers: builtin_providers(),
    }
}

fn merge_provider_config(
    base: &ProviderConfig,
    override_config: &ProviderConfig,
) -> ProviderConfig {
    ProviderConfig {
        r#type: override_config
            .r#type
            .clone()
            .or_else(|| base.r#type.clone()),
        base_url: trim_to_option(override_config.base_url.clone())
            .or_else(|| trim_to_option(base.base_url.clone())),
        api_key_env: trim_to_option(override_config.api_key_env.clone())
            .or_else(|| trim_to_option(base.api_key_env.clone())),
        api_key: trim_to_option(override_config.api_key.clone())
            .or_else(|| trim_to_option(base.api_key.clone())),
        default_model: trim_to_option(override_config.default_model.clone())
            .or_else(|| trim_to_option(base.default_model.clone())),
    }
}

fn merged_providers(config: &CozyConfig) -> BTreeMap<String, ProviderConfig> {
    let mut providers = builtin_providers();
    for (name, provider) in &config.providers {
        let existing = providers.get(name).cloned().unwrap_or_default();
        providers.insert(name.clone(), merge_provider_config(&existing, provider));
    }
    providers
}

fn validate_command_name(command: &str) -> Result<(), String> {
    match command {
        "prmsg" | "cmsg" => Ok(()),
        _ => Err(format!(
            "Unsupported AI command '{}'. Use 'prmsg' or 'cmsg'.",
            command
        )),
    }
}

pub fn validate_provider_name(name: &str) -> Result<&str, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Provider name cannot be empty.".to_string());
    }
    Ok(trimmed)
}

pub fn validate_provider_type(value: &str) -> Result<ProviderType, String> {
    match value.trim() {
        "gemini" => Ok(ProviderType::Gemini),
        "openai-compatible" => Ok(ProviderType::OpenaiCompatible),
        "opencode" => Ok(ProviderType::Opencode),
        other => Err(format!(
            "Unsupported provider type '{}'. Use 'gemini', 'openai-compatible', or 'opencode'.",
            other
        )),
    }
}

pub fn config_path() -> Option<PathBuf> {
    resolve_home_dir().map(|home| home.join(".cozyutils").join("config.json"))
}

fn parse_config(content: &str, path: &Path) -> Result<CozyConfig, String> {
    serde_json::from_str(content)
        .map_err(|error| format!("Failed to parse config {}: {}", path.display(), error))
}

fn write_config_file(path: &Path, config: &CozyConfig) -> Result<(), String> {
    let content = serde_json::to_string_pretty(config)
        .map_err(|error| format!("Failed to serialize config: {}", error))?;
    fs::write(path, format!("{}\n", content))
        .map_err(|error| format!("Failed to write config: {}", error))
}

pub fn load_config_or_default() -> Result<CozyConfig, String> {
    let path = config_path().ok_or_else(|| "Failed to resolve home directory.".to_string())?;
    match fs::read_to_string(&path) {
        Ok(content) => parse_config(&content, &path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(CozyConfig::default()),
        Err(error) => Err(format!(
            "Failed to read config {}: {}",
            path.display(),
            error
        )),
    }
}

pub fn resolve_command_settings(
    command: &str,
    provider_override: Option<&str>,
    model_override: Option<&str>,
) -> Result<ResolvedCommandSettings, String> {
    validate_command_name(command)?;

    let config = load_config_or_default()?;
    let command_defaults = config.defaults.commands.get(command);
    let provider_name = provider_override
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(|value| value.to_string())
    .or_else(|| command_defaults.and_then(|defaults| trim_to_option(defaults.provider.clone())))
    .or_else(|| trim_to_option(config.defaults.provider.clone()))
    .ok_or_else(|| {
      format!(
        "No provider configured for '{}'. Use --provider or -config --set-default-provider=NAME.",
        command
      )
    })?;

    let providers = merged_providers(&config);
    let provider = providers.get(&provider_name).cloned().ok_or_else(|| {
        format!(
            "Provider '{}' is not configured. Add it with -config before using it.",
            provider_name
        )
    })?;

    let model = model_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .or_else(|| command_defaults.and_then(|defaults| trim_to_option(defaults.model.clone())))
        .or_else(|| trim_to_option(provider.default_model.clone()))
        .ok_or_else(|| {
            format!(
                "No model configured for provider '{}'. Use --model or -config to set one.",
                provider_name
            )
        })?;

    let api_key = provider
        .api_key_env
        .as_deref()
        .and_then(|name| env::var(name).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| trim_to_option(provider.api_key.clone()));

    Ok(ResolvedCommandSettings {
        provider_name,
        provider,
        model,
        api_key,
    })
}

pub fn update_config<F>(mutator: F) -> Result<PathBuf, String>
where
    F: FnOnce(&mut CozyConfig) -> Result<(), String>,
{
    let path = config_path().ok_or_else(|| "Failed to resolve home directory.".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create config directory: {}", error))?;
    }

    let mut config = load_config_or_default()?;
    mutator(&mut config)?;
    write_config_file(&path, &config)?;
    Ok(path)
}

pub fn init_config(overwrite: bool) -> Result<PathBuf, String> {
    let path = config_path().ok_or_else(|| "Failed to resolve home directory.".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create config directory: {}", error))?;
    }

    if path.exists() && !overwrite {
        return Err(format!(
            "Config already exists at {}. Use -config --init --force to overwrite it.",
            path.display()
        ));
    }

    write_config_file(&path, &starter_config())?;
    Ok(path)
}

pub fn set_default_provider(
    config: &mut CozyConfig,
    provider_name: Option<&str>,
) -> Result<(), String> {
    config.defaults.provider = provider_name
        .map(validate_provider_name)
        .transpose()?
        .map(str::to_string);
    Ok(())
}

pub fn set_command_provider(
    config: &mut CozyConfig,
    command: &str,
    provider_name: Option<&str>,
) -> Result<(), String> {
    validate_command_name(command)?;
    let defaults = config
        .defaults
        .commands
        .entry(command.to_string())
        .or_default();
    defaults.provider = provider_name
        .map(validate_provider_name)
        .transpose()?
        .map(str::to_string);
    if defaults.provider.is_none() && defaults.model.is_none() {
        config.defaults.commands.remove(command);
    }
    Ok(())
}

pub fn set_command_model(
    config: &mut CozyConfig,
    command: &str,
    model: Option<&str>,
) -> Result<(), String> {
    validate_command_name(command)?;
    let defaults = config
        .defaults
        .commands
        .entry(command.to_string())
        .or_default();
    defaults.model = model
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if defaults.provider.is_none() && defaults.model.is_none() {
        config.defaults.commands.remove(command);
    }
    Ok(())
}

pub fn update_provider<F>(
    config: &mut CozyConfig,
    provider_name: &str,
    mutator: F,
) -> Result<(), String>
where
    F: FnOnce(&mut ProviderConfig) -> Result<(), String>,
{
    let name = validate_provider_name(provider_name)?.to_string();
    let base = builtin_providers().get(&name).cloned().unwrap_or_default();
    let provider = config.providers.entry(name.clone()).or_insert(base);
    mutator(provider)?;
    if provider.r#type.is_none()
        && provider.base_url.is_none()
        && provider.api_key_env.is_none()
        && provider.api_key.is_none()
        && provider.default_model.is_none()
    {
        config.providers.remove(&name);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CozyConfig, ProviderConfig, ProviderType, builtin_providers, resolve_command_settings,
        set_command_model, set_command_provider, set_default_provider, starter_config,
        update_provider, validate_provider_type,
    };

    #[test]
    fn builtin_providers_include_openai_compatible_defaults() {
        let providers = builtin_providers();

        assert_eq!(
            providers
                .get("openrouter")
                .and_then(|provider| provider.base_url.as_deref()),
            Some("https://openrouter.ai/api/v1")
        );
        assert_eq!(
            providers
                .get("deepseek")
                .and_then(|provider| provider.default_model.as_deref()),
            Some("deepseek-chat")
        );
    }

    #[test]
    fn validates_provider_types() {
        assert_eq!(
            validate_provider_type("openai-compatible").unwrap(),
            ProviderType::OpenaiCompatible
        );
        assert!(validate_provider_type("unknown").is_err());
    }

    #[test]
    fn command_defaults_are_removed_when_empty() {
        let mut config = CozyConfig::default();

        set_command_provider(&mut config, "prmsg", Some("openrouter")).unwrap();
        set_command_provider(&mut config, "prmsg", None).unwrap();

        assert!(!config.defaults.commands.contains_key("prmsg"));
    }

    #[test]
    fn provider_entries_are_removed_when_cleared() {
        let mut config = CozyConfig::default();

        update_provider(&mut config, "custom", |provider| {
            provider.r#type = Some(ProviderType::OpenaiCompatible);
            provider.base_url = Some("https://example.com/v1".to_string());
            Ok(())
        })
        .unwrap();

        update_provider(&mut config, "custom", |provider| {
            *provider = ProviderConfig::default();
            Ok(())
        })
        .unwrap();

        assert!(!config.providers.contains_key("custom"));
    }

    #[test]
    fn serialize_config_in_provider_first_shape() {
        let mut config = CozyConfig::default();
        set_default_provider(&mut config, Some("openrouter")).unwrap();
        set_command_provider(&mut config, "prmsg", Some("deepseek")).unwrap();
        set_command_model(&mut config, "prmsg", Some("deepseek-chat")).unwrap();

        update_provider(&mut config, "custom", |provider| {
            provider.r#type = Some(ProviderType::OpenaiCompatible);
            provider.base_url = Some("https://example.com/v1".to_string());
            provider.api_key_env = Some("EXAMPLE_API_KEY".to_string());
            provider.default_model = Some("example-model".to_string());
            Ok(())
        })
        .unwrap();

        let json = serde_json::to_string_pretty(&config).unwrap();

        assert!(json.contains("\"providers\""));
        assert!(json.contains("\"openai-compatible\""));
        assert!(!json.contains("gemini_api_key"));
        assert!(!json.contains("backend"));
    }

    #[test]
    fn resolve_command_settings_requires_configured_provider() {
        let error = resolve_command_settings("prmsg", Some("missing"), Some("model")).unwrap_err();

        assert!(error.contains("Provider 'missing' is not configured"));
    }

    #[test]
    fn starter_config_prefills_defaults_and_builtin_providers() {
        let config = starter_config();

        assert_eq!(config.defaults.provider.as_deref(), Some("openrouter"));
        assert_eq!(
            config
                .defaults
                .commands
                .get("cmsg")
                .and_then(|defaults| defaults.provider.as_deref()),
            Some("deepseek")
        );
        assert!(config.providers.contains_key("gemini"));
        assert!(config.providers.contains_key("openrouter"));
    }
}
