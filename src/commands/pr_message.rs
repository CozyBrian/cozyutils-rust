use std::env;
use std::path::Path;
use std::process::Command;

use crate::cli::args::parse_args;
use crate::utils::config::{load_gemini_api_key, write_config};
use crate::utils::fs::write_string;
use crate::utils::message::{copy_to_clipboard, generate_gemini_text, run_git_command};

const DEFAULT_BASE_REF: &str = "origin/dev";
const DEFAULT_MODEL: &str = "gemini-3-flash-preview";
const BASE_REF_FALLBACKS: &[&str] = &["origin/main", "origin/master", "main", "master", "dev"];

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
            .or_else(|_| {
                parsed
                    .options
                    .get("key")
                    .cloned()
                    .ok_or_else(|| env::VarError::NotPresent)
            })
            .map_err(|_| "Provide the API key via GEMINI_API_KEY or --key.".to_string())?;
        let path = write_config(&api_key)?;
        println!("prMessage - Config written to {}", path.display());
        return Ok(());
    }

    let api_key = load_gemini_api_key()?;

    let resolved_base_ref = resolve_base_ref(&base_ref)?;
    let status = run_git_command(&["status"], "status")?;
    let log = run_git_command(
        &["log", "--oneline", &format!("{}..HEAD", resolved_base_ref)],
        "log",
    )?;
    let diff_stat = run_git_command(
        &["diff", &format!("{}..HEAD", resolved_base_ref), "--stat"],
        "diff --stat",
    )?;
    let diff = run_git_command(&["diff", &format!("{}..HEAD", resolved_base_ref)], "diff")?;

    let prompt = [
        "Generate a PR description from the following git outputs.",
        "Return markdown formatted text suitable for a pull request description.",
        "Avoid quoting diffs verbatim unless needed for clarity.",
        "",
        "git status:",
        if status.is_empty() {
            "(no output)"
        } else {
            &status
        },
        "",
        &format!("git log --oneline {}..HEAD:", resolved_base_ref),
        if log.is_empty() { "(no commits)" } else { &log },
        "",
        &format!("git diff {}..HEAD --stat:", resolved_base_ref),
        if diff_stat.is_empty() {
            "(no changes)"
        } else {
            &diff_stat
        },
        "",
        &format!("git diff {}..HEAD:", resolved_base_ref),
        if diff.is_empty() {
            "(no changes)"
        } else {
            &diff
        },
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
