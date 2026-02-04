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
        if let Some(next_value) = next {
            if !next_value.starts_with("--") {
                options.insert(key.to_string(), next_value.to_string());
                index += 2;
                continue;
            }
        }

        options.insert(key.to_string(), "true".to_string());
        index += 1;
    }

    ParsedArgs {
        positional,
        options,
    }
}
