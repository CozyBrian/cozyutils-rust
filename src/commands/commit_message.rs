use crate::cli::args::parse_args;
use crate::utils::config::resolve_command_settings;
use crate::utils::fs::write_string;
use crate::utils::message::{copy_to_clipboard, generate_text, run_git_command};

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
    let should_commit = parsed.options.contains_key("commit");

    if parsed.options.contains_key("help") {
        println!(
            "Usage: -cmsg [--out=path] [--model=MODEL] [--provider=NAME] [--backend=NAME] [--clipboard] [--clipboard-only] [--commit]"
        );
        return Ok(());
    }

    let settings = resolve_command_settings("cmsg", provider, model)?;

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

    let commit_message_text = generate_text(&settings, &prompt)?;

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
        if clipboard && let Err(error) = copy_to_clipboard(&commit_message_text) {
            eprintln!("{}", error);
        }
        return Ok(());
    }

    if !clipboard_only {
        println!("{}", commit_message_text);
    }

    if clipboard && let Err(error) = copy_to_clipboard(&commit_message_text) {
        eprintln!("{}", error);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::split_commit_message;

    #[test]
    fn splits_subject_and_body_after_blank_line() {
        let parts = split_commit_message(
            "feat: improve svg conversion\n\n- preserve fill=\"none\"\n- exit non-zero on failure\n",
        );

        assert_eq!(parts.subject, "feat: improve svg conversion");
        assert_eq!(
            parts.body,
            "- preserve fill=\"none\"\n- exit non-zero on failure"
        );
    }

    #[test]
    fn returns_empty_parts_for_blank_message() {
        let parts = split_commit_message("  \n\n  ");

        assert!(parts.subject.is_empty());
        assert!(parts.body.is_empty());
    }

    #[test]
    fn discards_leading_body_text_before_first_blank_line() {
        let parts = split_commit_message(
            "fix: parser flag handling\nsummary line\n\n- keep current behavior\n- add tests\n",
        );

        assert_eq!(parts.subject, "fix: parser flag handling");
        assert_eq!(parts.body, "- keep current behavior\n- add tests");
    }
}
