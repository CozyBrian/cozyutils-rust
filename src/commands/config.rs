use crate::cli::args::parse_args;
use crate::utils::config::{
    ProviderType, config_path, init_config, load_config_or_default, set_command_model,
    set_command_provider, set_default_provider, update_config, update_provider,
    validate_provider_type,
};

fn masked_config_json() -> Result<String, String> {
    let mut config = load_config_or_default()?;
    for provider in config.providers.values_mut() {
        if provider.api_key.is_some() {
            provider.api_key = Some("***set***".to_string());
        }
    }
    serde_json::to_string_pretty(&config)
        .map_err(|error| format!("Failed to serialize config: {}", error))
}

fn split_pair<'a>(value: &'a str, option_name: &str) -> Result<(&'a str, &'a str), String> {
    let Some((left, right)) = value.split_once(':') else {
        return Err(format!("{} expects NAME:VALUE format.", option_name));
    };
    let left = left.trim();
    let right = right.trim();
    if left.is_empty() || right.is_empty() {
        return Err(format!("{} expects NAME:VALUE format.", option_name));
    }
    Ok((left, right))
}

fn parse_provider_type_assignment(value: &str) -> Result<(&str, ProviderType), String> {
    let (provider_name, provider_type) = split_pair(value, "--set-provider-type")?;
    Ok((provider_name, validate_provider_type(provider_type)?))
}

fn option_value<'a>(
    args: &'a std::collections::HashMap<String, String>,
    key: &str,
) -> Option<&'a str> {
    args.get(key).map(|value| value.as_str())
}

pub fn config_command(args: Vec<String>) -> Result<(), String> {
    let parsed = parse_args(&args);

    if parsed.options.contains_key("help") {
        println!(
            "Usage: -config [--init] [--force] [--show] [--path] [--set-default-provider=NAME] [--unset-default-provider] [--set-command-provider=COMMAND:NAME] [--unset-command-provider=COMMAND] [--set-command-model=COMMAND:MODEL] [--unset-command-model=COMMAND] [--set-provider-type=NAME:TYPE] [--set-provider-base-url=NAME:URL] [--set-provider-key=NAME:VALUE] [--unset-provider-key=NAME] [--set-provider-key-env=NAME:ENV] [--unset-provider-key-env=NAME] [--set-provider-model=NAME:MODEL] [--unset-provider-model=NAME]"
        );
        return Ok(());
    }

    if parsed.options.contains_key("init") {
        let path = init_config(parsed.options.contains_key("force"))?;
        println!("config - Initialized {}", path.display());
        return Ok(());
    }

    if parsed.options.contains_key("path") {
        let path = config_path().ok_or_else(|| "Failed to resolve home directory.".to_string())?;
        println!("{}", path.display());
        return Ok(());
    }

    if parsed.options.contains_key("show") {
        println!("{}", masked_config_json()?);
        return Ok(());
    }

    let set_default_provider_value = option_value(&parsed.options, "set-default-provider");
    let unset_default_provider = parsed.options.contains_key("unset-default-provider");
    if set_default_provider_value.is_some() && unset_default_provider {
        return Err(
            "Use either --set-default-provider or --unset-default-provider, not both.".to_string(),
        );
    }

    let set_command_provider_value = option_value(&parsed.options, "set-command-provider");
    let unset_command_provider_value = option_value(&parsed.options, "unset-command-provider");
    if set_command_provider_value.is_some() && unset_command_provider_value.is_some() {
        return Err(
      "Use either --set-command-provider or --unset-command-provider for a command, not both."
        .to_string(),
    );
    }

    let set_command_model_value = option_value(&parsed.options, "set-command-model");
    let unset_command_model_value = option_value(&parsed.options, "unset-command-model");
    if set_command_model_value.is_some() && unset_command_model_value.is_some() {
        return Err(
            "Use either --set-command-model or --unset-command-model for a command, not both."
                .to_string(),
        );
    }

    let set_provider_key_value = option_value(&parsed.options, "set-provider-key");
    let unset_provider_key_value = option_value(&parsed.options, "unset-provider-key");
    if set_provider_key_value.is_some() && unset_provider_key_value.is_some() {
        return Err(
            "Use either --set-provider-key or --unset-provider-key for a provider, not both."
                .to_string(),
        );
    }

    let set_provider_key_env_value = option_value(&parsed.options, "set-provider-key-env");
    let unset_provider_key_env_value = option_value(&parsed.options, "unset-provider-key-env");
    if set_provider_key_env_value.is_some() && unset_provider_key_env_value.is_some() {
        return Err(
      "Use either --set-provider-key-env or --unset-provider-key-env for a provider, not both."
        .to_string(),
    );
    }

    let set_provider_model_value = option_value(&parsed.options, "set-provider-model");
    let unset_provider_model_value = option_value(&parsed.options, "unset-provider-model");
    if set_provider_model_value.is_some() && unset_provider_model_value.is_some() {
        return Err(
            "Use either --set-provider-model or --unset-provider-model for a provider, not both."
                .to_string(),
        );
    }

    let has_action = set_default_provider_value.is_some()
        || unset_default_provider
        || set_command_provider_value.is_some()
        || unset_command_provider_value.is_some()
        || set_command_model_value.is_some()
        || unset_command_model_value.is_some()
        || option_value(&parsed.options, "set-provider-type").is_some()
        || option_value(&parsed.options, "set-provider-base-url").is_some()
        || set_provider_key_value.is_some()
        || unset_provider_key_value.is_some()
        || set_provider_key_env_value.is_some()
        || unset_provider_key_env_value.is_some()
        || set_provider_model_value.is_some()
        || unset_provider_model_value.is_some();

    if !has_action {
        return Err(
      "No config action specified. Use --show, --path, or one of the --set/--unset provider options."
        .to_string(),
    );
    }

    let path = update_config(|config| {
        if let Some(provider_name) = set_default_provider_value {
            set_default_provider(config, Some(provider_name))?;
        } else if unset_default_provider {
            set_default_provider(config, None)?;
        }

        if let Some(value) = set_command_provider_value {
            let (command, provider_name) = split_pair(value, "--set-command-provider")?;
            set_command_provider(config, command, Some(provider_name))?;
        }

        if let Some(command) = unset_command_provider_value {
            set_command_provider(config, command, None)?;
        }

        if let Some(value) = set_command_model_value {
            let (command, model) = split_pair(value, "--set-command-model")?;
            set_command_model(config, command, Some(model))?;
        }

        if let Some(command) = unset_command_model_value {
            set_command_model(config, command, None)?;
        }

        if let Some(value) = option_value(&parsed.options, "set-provider-type") {
            let (provider_name, provider_type) = parse_provider_type_assignment(value)?;
            update_provider(config, provider_name, |provider| {
                provider.r#type = Some(provider_type);
                Ok(())
            })?;
        }

        if let Some(value) = option_value(&parsed.options, "set-provider-base-url") {
            let (provider_name, base_url) = split_pair(value, "--set-provider-base-url")?;
            update_provider(config, provider_name, |provider| {
                provider.base_url = Some(base_url.to_string());
                Ok(())
            })?;
        }

        if let Some(value) = set_provider_key_value {
            let (provider_name, api_key) = split_pair(value, "--set-provider-key")?;
            update_provider(config, provider_name, |provider| {
                provider.api_key = Some(api_key.to_string());
                Ok(())
            })?;
        }

        if let Some(provider_name) = unset_provider_key_value {
            update_provider(config, provider_name, |provider| {
                provider.api_key = None;
                Ok(())
            })?;
        }

        if let Some(value) = set_provider_key_env_value {
            let (provider_name, env_name) = split_pair(value, "--set-provider-key-env")?;
            update_provider(config, provider_name, |provider| {
                provider.api_key_env = Some(env_name.to_string());
                Ok(())
            })?;
        }

        if let Some(provider_name) = unset_provider_key_env_value {
            update_provider(config, provider_name, |provider| {
                provider.api_key_env = None;
                Ok(())
            })?;
        }

        if let Some(value) = set_provider_model_value {
            let (provider_name, model) = split_pair(value, "--set-provider-model")?;
            update_provider(config, provider_name, |provider| {
                provider.default_model = Some(model.to_string());
                Ok(())
            })?;
        }

        if let Some(provider_name) = unset_provider_model_value {
            update_provider(config, provider_name, |provider| {
                provider.default_model = None;
                Ok(())
            })?;
        }

        Ok(())
    })?;

    println!("config - Updated {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::split_pair;
    use crate::cli::args::parse_args;

    #[test]
    fn split_pair_requires_name_value_format() {
        assert!(split_pair("value", "--flag").is_err());
        assert!(split_pair(":value", "--flag").is_err());
        assert!(split_pair("name:", "--flag").is_err());
        assert_eq!(
            split_pair("prmsg:openrouter", "--flag").unwrap(),
            ("prmsg", "openrouter")
        );
    }

    #[test]
    fn init_is_parsed_as_boolean_flag() {
        let parsed = parse_args(&["--init".to_string(), "--force".to_string()]);

        assert_eq!(parsed.options.get("init"), Some(&"true".to_string()));
        assert_eq!(parsed.options.get("force"), Some(&"true".to_string()));
    }
}
