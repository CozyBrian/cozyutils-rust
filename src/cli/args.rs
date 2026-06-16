use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct ParsedArgs {
    pub positional: Vec<String>,
    pub options: HashMap<String, String>,
}

fn is_boolean_flag(flag: &str) -> bool {
    matches!(
        flag,
        "dry-run"
            | "force"
            | "no-move"
            | "help"
            | "clipboard"
            | "clipboard-only"
            | "setup"
            | "commit"
    ) || flag == "copy"
}

pub fn parse_args(args: &[String]) -> ParsedArgs {
    let mut positional = Vec::new();
    let mut options = HashMap::new();

    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];

        if !arg.starts_with("--") {
            positional.push(arg.to_string());
            index += 1;
            continue;
        }

        let trimmed = &arg[2..];
        let mut parts = trimmed.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let inline_value = parts.next();

        if let Some(value) = inline_value {
            options.insert(key.to_string(), value.to_string());
            index += 1;
            continue;
        }

        if is_boolean_flag(key) {
            options.insert(key.to_string(), "true".to_string());
            index += 1;
            continue;
        }

        let next = args.get(index + 1);
        if let Some(next_value) = next
            && !next_value.starts_with("--")
        {
            options.insert(key.to_string(), next_value.to_string());
            index += 2;
            continue;
        }

        options.insert(key.to_string(), "true".to_string());
        index += 1;
    }

    ParsedArgs {
        positional,
        options,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parses_inline_and_separate_option_values() {
        let parsed = parse_args(&args(&[
            "input",
            "--ext=.svg,.png",
            "--out",
            "index.ts",
            "other",
        ]));

        assert_eq!(parsed.positional, vec!["input", "other"]);
        assert_eq!(parsed.options.get("ext"), Some(&".svg,.png".to_string()));
        assert_eq!(parsed.options.get("out"), Some(&"index.ts".to_string()));
    }

    #[test]
    fn parses_boolean_flags_without_consuming_positionals() {
        let parsed = parse_args(&args(&["--dry-run", "icons", "--copy", "index.ts"]));

        assert_eq!(parsed.positional, vec!["icons", "index.ts"]);
        assert_eq!(parsed.options.get("dry-run"), Some(&"true".to_string()));
        assert_eq!(parsed.options.get("copy"), Some(&"true".to_string()));
    }

    #[test]
    fn falls_back_to_true_when_option_value_is_missing() {
        let parsed = parse_args(&args(&["--base", "--clipboard"]));

        assert_eq!(parsed.options.get("base"), Some(&"true".to_string()));
        assert_eq!(parsed.options.get("clipboard"), Some(&"true".to_string()));
    }
}
