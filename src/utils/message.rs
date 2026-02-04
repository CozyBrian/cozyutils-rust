use std::io::Write;
use std::process::Command;

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
