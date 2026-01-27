use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::cli::args::parse_args;
use crate::utils::fs::write_string;

const DEFAULT_BASE_REF: &str = "origin/dev";
const DEFAULT_MODEL: &str = "gemini-3-flash-preview";
const BASE_REF_FALLBACKS: &[&str] = &[
  "origin/main",
  "origin/master",
  "main",
  "master",
  "dev",
];

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

#[derive(Debug, Deserialize, Serialize)]
struct CozyConfig {
  gemini_api_key: Option<String>,
}

fn log_run(command: &str) {
  println!("Running '{}'", command);
}

fn run_git_command(args: &[&str], label: &str) -> Result<String, String> {
  log_run(&format!("git {}", args.join(" ")));
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

  Ok(String::from_utf8_lossy(&output.stdout).trim_end().to_string())
}

fn check_git_ref(ref_name: &str) -> bool {
  Command::new("git")
    .args(["rev-parse", "--verify", ref_name])
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
    .map(|status| status.success())
    .unwrap_or(false)
}

fn resolve_base_ref(base_ref: &str) -> Result<String, String> {
  if check_git_ref(base_ref) {
    return Ok(base_ref.to_string());
  }

  if base_ref != DEFAULT_BASE_REF {
    return Err(format!(
      "Base ref '{}' not found. Use --base to specify a valid ref.",
      base_ref
    ));
  }

  for fallback in BASE_REF_FALLBACKS {
    if check_git_ref(fallback) {
      println!(
        "Base ref '{}' not found. Falling back to '{}'.",
        base_ref, fallback
      );
      return Ok((*fallback).to_string());
    }
  }

  Err(format!(
    "Base ref '{}' not found. Run 'git fetch' or set --base to a valid ref.",
    base_ref
  ))
}

fn generate_gemini_text(api_key: &str, model: &str, prompt: &str) -> Result<String, String> {
  log_run(&format!("Generating ({})", model));
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
    let message = response
      .into_string()
      .unwrap_or_else(|_| "".to_string());
    return Err(format!(
      "Gemini API request failed: {} {}",
      status,
      message
    ));
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

fn copy_to_clipboard(text: &str) -> Result<String, String> {
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

  Err(
    "No clipboard command available. Install pbcopy, wl-copy, xclip, or xsel.".to_string(),
  )
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

fn load_config_api_key() -> Option<String> {
  let home = resolve_home_dir()?;
  let path = home.join(".cozyutils").join("config.json");
  let content = fs::read_to_string(path).ok()?;
  let config: CozyConfig = serde_json::from_str(&content).ok()?;
  config
    .gemini_api_key
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
}

fn config_path() -> Option<PathBuf> {
  resolve_home_dir().map(|home| home.join(".cozyutils").join("config.json"))
}

fn write_config(api_key: &str) -> Result<PathBuf, String> {
  let path = config_path().ok_or_else(|| "Failed to resolve home directory.".to_string())?;
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)
      .map_err(|error| format!("Failed to create config directory: {}", error))?;
  }

  let config = CozyConfig {
    gemini_api_key: Some(api_key.to_string()),
  };
  let content = serde_json::to_string_pretty(&config)
    .map_err(|error| format!("Failed to serialize config: {}", error))?;
  fs::write(&path, format!("{}\n", content))
    .map_err(|error| format!("Failed to write config: {}", error))?;
  Ok(path)
}

pub fn pr_message(args: Vec<String>) -> Result<(), String> {
  let parsed = parse_args(&args);
  let base_ref = parsed
    .options
    .get("base")
    .cloned()
    .or_else(|| parsed.positional.get(0).cloned())
    .unwrap_or_else(|| DEFAULT_BASE_REF.to_string());
  let output_path = parsed.options.get("out").cloned().unwrap_or_default();
  let model = parsed
    .options
    .get("model")
    .cloned()
    .unwrap_or_else(|| DEFAULT_MODEL.to_string());
  let clipboard_only = parsed.options.get("clipboard-only").is_some();
  let clipboard = clipboard_only
    || parsed.options.get("clipboard").is_some()
    || parsed.options.get("copy").is_some();
  let setup = parsed.options.get("setup").is_some();

  if parsed.options.get("help").is_some() {
    println!(
      "Usage: -prmsg [--base=origin/dev] [--out=path] [--model=gemini-3-flash-preview] [--clipboard] [--clipboard-only] [--setup]"
    );
    return Ok(());
  }

  if setup {
    let api_key = env::var("GEMINI_API_KEY")
      .or_else(|_| parsed.options.get("key").cloned().ok_or_else(|| env::VarError::NotPresent))
      .map_err(|_| "Provide the API key via GEMINI_API_KEY or --key.".to_string())?;
    let path = write_config(&api_key)?;
    println!("prMessage - Config written to {}", path.display());
    return Ok(());
  }

  let api_key = env::var("GEMINI_API_KEY")
    .ok()
    .or_else(load_config_api_key)
    .ok_or_else(|| {
      "Missing GEMINI_API_KEY environment variable or ~/.cozyutils/config.json entry."
        .to_string()
    })?;

  let resolved_base_ref = resolve_base_ref(&base_ref)?;
  let status = run_git_command(&["status"], "status")?;
  let log = run_git_command(&["log", "--oneline", &format!("{}..HEAD", resolved_base_ref)], "log")?;
  let diff_stat = run_git_command(&[
    "diff",
    &format!("{}..HEAD", resolved_base_ref),
    "--stat",
  ], "diff --stat")?;
  let diff = run_git_command(&[
    "diff",
    &format!("{}..HEAD", resolved_base_ref),
  ], "diff")?;

  let prompt = [
    "Generate a PR description from the following git outputs.",
    "Return markdown formatted text suitable for a pull request description.",
    "Avoid quoting diffs verbatim unless needed for clarity.",
    "",
    "git status:",
    if status.is_empty() { "(no output)" } else { &status },
    "",
    &format!("git log --oneline {}..HEAD:", resolved_base_ref),
    if log.is_empty() { "(no commits)" } else { &log },
    "",
    &format!("git diff {}..HEAD --stat:", resolved_base_ref),
    if diff_stat.is_empty() { "(no changes)" } else { &diff_stat },
    "",
    &format!("git diff {}..HEAD:", resolved_base_ref),
    if diff.is_empty() { "(no changes)" } else { &diff },
  ]
  .join("\n");

  let pr_message_text = generate_gemini_text(&api_key, &model, &prompt)?;

  if !output_path.is_empty() && !clipboard_only {
    let path = Path::new(&output_path);
    write_string(path, &format!("{}\n", pr_message_text))?;
    println!("prMessage - Done! Wrote {}", path.display());
    if clipboard {
      match copy_to_clipboard(&pr_message_text) {
        Ok(label) => println!("prMessage - Copied to clipboard ({})", label),
        Err(error) => println!("{}", error),
      }
    }
    return Ok(());
  }

  if !clipboard_only {
    println!("{}", pr_message_text);
  }

  if clipboard {
    match copy_to_clipboard(&pr_message_text) {
      Ok(label) => println!("prMessage - Copied to clipboard ({})", label),
      Err(error) => println!("{}", error),
    }
  }

  Ok(())
}
