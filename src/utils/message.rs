use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde_json::json;

use crate::utils::config::{ProviderConfig, ProviderType, ResolvedCommandSettings};

const CLIPBOARD_COMMANDS: &[(&[&str], &str)] = &[
    (&["pbcopy"], "pbcopy"),
    (&["wl-copy"], "wl-copy"),
    (&["xclip", "-selection", "clipboard"], "xclip"),
    (&["xsel", "--clipboard", "--input"], "xsel"),
    (&["clip"], "clip"),
];

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    error: Option<GeminiError>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    parts: Option<Vec<GeminiPart>>,
}

#[derive(Debug, Deserialize)]
struct GeminiPart {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiError {
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiCompatibleResponse {
    choices: Option<Vec<OpenAiChoice>>,
    error: Option<OpenAiError>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: Option<OpenAiMessage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessage {
    content: Option<OpenAiContent>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OpenAiContent {
    Text(String),
    Parts(Vec<OpenAiContentPart>),
}

#[derive(Debug, Deserialize)]
struct OpenAiContentPart {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiError {
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenCodeEvent {
    #[serde(rename = "type")]
    event_type: String,
    part: Option<OpenCodePart>,
}

#[derive(Debug, Deserialize)]
struct OpenCodePart {
    text: Option<String>,
}

pub fn run_git_command(args: &[&str], label: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|error| format!("git {} failed: {}", label, error))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let message = if !stderr.is_empty() { stderr } else { stdout };
        let suffix = if message.is_empty() {
            "".to_string()
        } else {
            format!(": {}", message)
        };
        return Err(format!("git {} failed{}", label, suffix));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string())
}

pub fn generate_gemini_text(api_key: &str, model: &str, prompt: &str) -> Result<String, String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let body = json!({
      "contents": [{ "role": "user", "parts": [{ "text": prompt }] }],
      "generationConfig": { "temperature": 0.2 }
    });

    let response = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|error| format!("Gemini API request failed: {}", error))?;

    let data: GeminiResponse = response
        .into_json()
        .map_err(|error| format!("Gemini API response parse failed: {}", error))?;

    if let Some(error) = data.error
        && let Some(message) = error.message
    {
        return Err(format!("Gemini API error: {}", message));
    }

    let text = data
        .candidates
        .unwrap_or_default()
        .into_iter()
        .flat_map(|candidate| candidate.content)
        .flat_map(|content| content.parts.unwrap_or_default())
        .filter_map(|part| part.text)
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string();

    if text.is_empty() {
        return Err("Gemini response was empty.".to_string());
    }

    Ok(text)
}

fn provider_type(provider_name: &str, provider: &ProviderConfig) -> Result<ProviderType, String> {
    provider.r#type.clone().ok_or_else(|| {
        format!(
            "Provider '{}' is missing a type. Set one with -config --set-provider-type={}:TYPE.",
            provider_name, provider_name
        )
    })
}

fn openai_content_to_text(content: OpenAiContent) -> String {
    match content {
        OpenAiContent::Text(text) => text,
        OpenAiContent::Parts(parts) => parts
            .into_iter()
            .filter_map(|part| part.text)
            .collect::<Vec<_>>()
            .join(""),
    }
}

pub fn generate_openai_compatible_text(
    provider_name: &str,
    provider: &ProviderConfig,
    api_key: &str,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    let base_url = provider.base_url.as_deref().ok_or_else(|| {
        format!(
            "Provider '{}' is missing a base_url. Set one with -config --set-provider-base-url={}:URL.",
            provider_name, provider_name
        )
    })?;
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let body = json!({
      "model": model,
      "messages": [{ "role": "user", "content": prompt }],
      "temperature": 0.2,
    });

    let response = ureq::post(&url)
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {}", api_key))
        .send_json(body)
        .map_err(|error| format!("{} API request failed: {}", provider_name, error))?;

    let data: OpenAiCompatibleResponse = response
        .into_json()
        .map_err(|error| format!("{} API response parse failed: {}", provider_name, error))?;

    if let Some(error) = data.error
        && let Some(message) = error.message
    {
        return Err(format!("{} API error: {}", provider_name, message));
    }

    let text = data
        .choices
        .unwrap_or_default()
        .into_iter()
        .filter_map(|choice| choice.message)
        .filter_map(|message| message.content)
        .map(openai_content_to_text)
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string();

    if text.is_empty() {
        return Err(format!("{} response was empty.", provider_name));
    }

    Ok(text)
}

pub fn generate_opencode_text(model: &str, prompt: &str) -> Result<String, String> {
    let prompt_path = write_temp_prompt(prompt)?;
    let prompt_path_str = prompt_path.to_str().ok_or_else(|| {
        format!(
            "Temp prompt path is not valid UTF-8: {}",
            prompt_path.display()
        )
    })?;
    let output = Command::new("opencode")
        .args([
            "run",
            "--format",
            "json",
            "--model",
            model,
            "--file",
            prompt_path_str,
            "--",
            "Read the attached file and follow its instructions. Output only the requested response with no extra commentary.",
        ])
        .output();
    let _ = std::fs::remove_file(&prompt_path);

    let output = match output {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(
                "opencode is not installed. Install it from https://opencode.ai/install"
                    .to_string(),
            );
        }
        Err(error) => return Err(format!("opencode run failed: {}", error)),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let message = if !stderr.is_empty() { stderr } else { stdout };
        let suffix = if message.is_empty() {
            "".to_string()
        } else {
            format!(": {}", message)
        };
        return Err(format!("opencode run failed{}", suffix));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let text = stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<OpenCodeEvent>(line).ok())
        .filter(|event| event.event_type == "text")
        .filter_map(|event| event.part.and_then(|part| part.text))
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string();

    if text.is_empty() {
        return Err("OpenCode response was empty.".to_string());
    }

    Ok(text)
}

pub fn generate_text(settings: &ResolvedCommandSettings, prompt: &str) -> Result<String, String> {
    let provider_type = provider_type(&settings.provider_name, &settings.provider)?;

    match provider_type {
        ProviderType::Gemini => {
            let api_key = settings.api_key.as_deref().ok_or_else(|| {
                let env_name = settings
                    .provider
                    .api_key_env
                    .as_deref()
                    .unwrap_or("GEMINI_API_KEY");
                format!(
                    "Provider '{}' requires an API key. Set {} or store api_key in ~/.cozyutils/config.json.",
                    settings.provider_name, env_name
                )
            })?;
            generate_gemini_text(api_key, &settings.model, prompt)
        }
        ProviderType::OpenaiCompatible => {
            let api_key = settings.api_key.as_deref().ok_or_else(|| {
                if let Some(env_name) = settings.provider.api_key_env.as_deref() {
                    format!(
                        "Provider '{}' requires an API key. Set {} or store api_key in ~/.cozyutils/config.json.",
                        settings.provider_name, env_name
                    )
                } else {
                    format!(
                        "Provider '{}' requires an API key. Set api_key_env or store api_key in ~/.cozyutils/config.json.",
                        settings.provider_name
                    )
                }
            })?;
            generate_openai_compatible_text(
                &settings.provider_name,
                &settings.provider,
                api_key,
                &settings.model,
                prompt,
            )
        }
        ProviderType::Opencode => generate_opencode_text(&settings.model, prompt),
    }
}

fn write_temp_prompt(prompt: &str) -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("Failed to generate temp filename: {}", error))?
        .as_millis();
    let path = std::env::temp_dir().join(format!(
        "cozyutils-opencode-prompt-{}-{}.txt",
        std::process::id(),
        timestamp
    ));
    std::fs::write(&path, prompt)
        .map_err(|error| format!("Failed to write temp prompt: {}", error))?;
    Ok(path)
}

pub fn copy_to_clipboard(text: &str) -> Result<String, String> {
    for (command, label) in CLIPBOARD_COMMANDS {
        let mut child = match Command::new(command[0])
            .args(&command[1..])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => continue,
        };

        if let Some(mut stdin) = child.stdin.take()
            && stdin.write_all(text.as_bytes()).is_err()
        {
            continue;
        }

        if child.wait().map(|status| status.success()).unwrap_or(false) {
            return Ok((*label).to_string());
        }
    }

    Err("No clipboard command available. Install pbcopy, wl-copy, xclip, or xsel.".to_string())
}

#[cfg(test)]
mod tests {
    use super::{OpenAiCompatibleResponse, OpenAiContent, generate_openai_compatible_text};
    use crate::utils::config::{ProviderConfig, ProviderType};

    #[test]
    fn parses_string_openai_content() {
        let data: OpenAiCompatibleResponse =
            serde_json::from_str(r#"{"choices":[{"message":{"content":"hello"}}]}"#).unwrap();

        let text = data
            .choices
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
            .message
            .unwrap()
            .content
            .unwrap();

        assert!(matches!(text, OpenAiContent::Text(value) if value == "hello"));
    }

    #[test]
    fn parses_parts_openai_content() {
        let data: OpenAiCompatibleResponse = serde_json::from_str(
            r#"{"choices":[{"message":{"content":[{"text":"hello "},{"text":"world"}]}}]}"#,
        )
        .unwrap();

        let text = data
            .choices
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
            .message
            .unwrap()
            .content
            .unwrap();

        match text {
            OpenAiContent::Parts(parts) => {
                assert_eq!(parts.len(), 2);
                assert_eq!(parts[0].text.as_deref(), Some("hello "));
                assert_eq!(parts[1].text.as_deref(), Some("world"));
            }
            _ => panic!("expected parts content"),
        }
    }

    #[test]
    fn requires_base_url_for_openai_compatible_provider() {
        let error = generate_openai_compatible_text(
            "custom",
            &ProviderConfig {
                r#type: Some(ProviderType::OpenaiCompatible),
                base_url: None,
                api_key_env: None,
                api_key: None,
                default_model: None,
            },
            "key",
            "model",
            "prompt",
        )
        .unwrap_err();

        assert!(error.contains("missing a base_url"));
    }
}
