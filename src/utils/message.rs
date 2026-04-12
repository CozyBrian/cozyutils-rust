use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde_json::json;

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

    let status = response.status();
    if status >= 400 {
        let message = response.into_string().unwrap_or_else(|_| "".to_string());
        return Err(format!("Gemini API request failed: {} {}", status, message));
    }

    let data: GeminiResponse = response
        .into_json()
        .map_err(|error| format!("Gemini API response parse failed: {}", error))?;

    if let Some(error) = data.error {
        if let Some(message) = error.message {
            return Err(format!("Gemini API error: {}", message));
        }
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

pub fn generate_opencode_text(model: &str, prompt: &str) -> Result<String, String> {
    let prompt_path = write_temp_prompt(prompt)?;
    let output = Command::new("opencode")
        .args([
            "run",
            "--format",
            "json",
            "--model",
            model,
            "--file",
            prompt_path.to_str().unwrap_or(""),
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

pub fn generate_text(
    backend: &str,
    api_key: Option<&str>,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    match backend {
        "gemini" => {
            let api_key = api_key.ok_or_else(|| {
                "Missing GEMINI_API_KEY environment variable or ~/.cozyutils/config.json entry."
                    .to_string()
            })?;
            generate_gemini_text(api_key, model, prompt)
        }
        "opencode" => generate_opencode_text(model, prompt),
        _ => Err(format!(
            "Unsupported backend '{}'. Use 'gemini' or 'opencode'.",
            backend
        )),
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

        if let Some(mut stdin) = child.stdin.take() {
            if stdin.write_all(text.as_bytes()).is_err() {
                continue;
            }
        }

        if child.wait().map(|status| status.success()).unwrap_or(false) {
            return Ok((*label).to_string());
        }
    }

    Err("No clipboard command available. Install pbcopy, wl-copy, xclip, or xsel.".to_string())
}
