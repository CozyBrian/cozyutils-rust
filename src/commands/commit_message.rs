use crate::cli::args::parse_args;
use crate::utils::config::load_gemini_api_key;
use crate::utils::fs::write_string;
use crate::utils::message::{copy_to_clipboard, generate_gemini_text, run_git_command};

const DEFAULT_MODEL: &str = "gemini-3-flash-preview";

#[derive(Debug)]
struct CommitParts {
    subject: String,
    body: String,
}

fn split_commit_message(text: &str) -> CommitParts {
    let mut lines = text.trim().lines();
    let subject = lines.next().unwrap_or("").trim().to_string();
    if subject.is_empty() {
        return CommitParts {
            subject: String::new(),
            body: String::new(),
        };
    }

    let mut body_lines: Vec<String> = lines.map(|line| line.to_string()).collect();
    if let Some(index) = body_lines.iter().position(|line| line.trim().is_empty()) {
        body_lines = body_lines.into_iter().skip(index + 1).collect();
    }

    CommitParts {
        subject,
        body: body_lines.join("\n").trim().to_string(),
    }
}

pub fn commit_message(args: Vec<String>) -> Result<(), String> {
    let parsed = parse_args(&args);
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
    let should_commit = parsed.options.get("commit").is_some();

    if parsed.options.get("help").is_some() {
        println!(
      "Usage: -cmsg [--out=path] [--model=gemini-3-flash-preview] [--clipboard] [--clipboard-only] [--commit]"
    );
        return Ok(());
    }

    let api_key = load_gemini_api_key()?;

    let status = run_git_command(&["status"], "status")?;
    let diff_stat = run_git_command(&["diff", "--cached", "--stat"], "diff --cached --stat")?;
    let diff = run_git_command(&["diff", "--cached"], "diff --cached")?;

    let prompt = [
    "Generate a Conventional Commit message from the staged git changes below.",
    "Output ONLY the commit message with no extra commentary.",
    "Format:",
    "- First line: conventional commit subject (e.g., feat:, fix:, chore:, docs:, refactor:, test:)",
    "- Blank line",
    "- 2 to 5 bullet points describing the changes",
    "Avoid quoting diffs verbatim unless needed for clarity.",
    "",
    "git status:",
    if status.is_empty() { "(no output)" } else { &status },
    "",
    "git diff --cached --stat:",
    if diff_stat.is_empty() { "(no changes)" } else { &diff_stat },
    "",
    "git diff --cached:",
    if diff.is_empty() { "(no changes)" } else { &diff },
  ]
  .join("\n");

    let commit_message_text = generate_gemini_text(&api_key, &model, &prompt)?;

    if should_commit {
        let parts = split_commit_message(&commit_message_text);
        if parts.subject.is_empty() {
            return Err("Commit message subject was empty.".to_string());
        }
        let mut args = vec!["commit", "-m", parts.subject.as_str()];
        if !parts.body.is_empty() {
            args.push("-m");
            args.push(parts.body.as_str());
        }
        run_git_command(&args, "commit")?;
    }

    if !output_path.is_empty() && !clipboard_only {
        write_string(
            std::path::Path::new(&output_path),
            &format!("{}\n", commit_message_text),
        )?;
        if clipboard {
            if let Err(error) = copy_to_clipboard(&commit_message_text) {
                println!("{}", error);
            }
        }
        return Ok(());
    }

    if !clipboard_only {
        println!("{}", commit_message_text);
    }

    if clipboard {
        if let Err(error) = copy_to_clipboard(&commit_message_text) {
            println!("{}", error);
        }
    }

    Ok(())
}
