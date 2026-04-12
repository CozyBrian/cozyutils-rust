use crate::cli::args::parse_args;
use crate::utils::config::{config_path, load_config_or_default, update_config};

fn masked_config_json() -> Result<String, String> {
    let mut config = load_config_or_default();
    if config.gemini_api_key.is_some() {
        config.gemini_api_key = Some("***set***".to_string());
    }
    serde_json::to_string_pretty(&config)
        .map_err(|error| format!("Failed to serialize config: {}", error))
}

fn validate_backend(backend: &str) -> Result<&str, String> {
    match backend {
        "gemini" | "opencode" => Ok(backend),
        _ => Err(format!(
            "Unsupported backend '{}'. Use 'gemini' or 'opencode'.",
            backend
        )),
    }
}

pub fn config_command(args: Vec<String>) -> Result<(), String> {
    let parsed = parse_args(&args);

    if parsed.options.contains_key("help") {
        println!(
            "Usage: -config [--show] [--path] [--set-backend=gemini|opencode] [--unset-backend] [--set-key=VALUE] [--unset-key]"
        );
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

    let set_key = parsed.options.get("set-key").map(|value| value.as_str());
    let unset_key = parsed.options.contains_key("unset-key");
    if set_key.is_some() && unset_key {
        return Err("Use either --set-key or --unset-key, not both.".to_string());
    }

    let set_backend = parsed
        .options
        .get("set-backend")
        .map(|value| value.as_str());
    let unset_backend = parsed.options.contains_key("unset-backend");
    if set_backend.is_some() && unset_backend {
        return Err("Use either --set-backend or --unset-backend, not both.".to_string());
    }

    let key_update = if let Some(value) = set_key {
        Some(Some(value))
    } else if unset_key {
        Some(None)
    } else {
        None
    };

    let backend_update = if let Some(value) = set_backend {
        Some(Some(validate_backend(value)?))
    } else if unset_backend {
        Some(None)
    } else {
        None
    };

    if key_update.is_none() && backend_update.is_none() {
        return Err(
            "No config action specified. Use --show, --path, --set-backend, --unset-backend, --set-key, or --unset-key."
                .to_string(),
        );
    }

    let path = update_config(key_update, backend_update)?;
    println!("config - Updated {}", path.display());
    Ok(())
}
