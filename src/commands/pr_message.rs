use std::path::Path;
use std::process::Command;

use crate::cli::args::parse_args;
use crate::utils::config::resolve_command_settings;
use crate::utils::fs::write_string;
use crate::utils::message::{copy_to_clipboard, generate_text, run_git_command};

const DEFAULT_BASE_REF: &str = "origin/dev";
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
        .or_else(|| parsed.positional.first().cloned())
        .unwrap_or_else(|| DEFAULT_BASE_REF.to_string());
    let output_path = parsed.options.get("out").cloned().unwrap_or_default();
    let provider = parsed
        .options
        .get("provider")
        .or_else(|| parsed.options.get("backend"))
        .map(|value| value.as_str());
    let model = parsed.options.get("model").map(|value| value.as_str());
    let clipboard_only = parsed.options.contains_key("clipboard-only");
    let clipboard = clipboard_only
        || parsed.options.contains_key("clipboard")
        || parsed.options.contains_key("copy");

    if parsed.options.contains_key("help") {
        println!(
            "Usage: -prmsg [--base=origin/dev] [--out=path] [--model=MODEL] [--provider=NAME] [--backend=NAME] [--clipboard] [--clipboard-only]"
        );
        return Ok(());
    }

    if parsed.options.contains_key("setup") {
        return Err(
            "-prmsg --setup is no longer supported. Use -config to manage providers and defaults."
                .to_string(),
        );
    }

    let settings = resolve_command_settings("prmsg", provider, model)?;

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

    let pr_message_text = generate_text(&settings, &prompt)?;

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
