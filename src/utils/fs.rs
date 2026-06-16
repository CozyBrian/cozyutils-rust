use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

static SPACE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" {2,}").unwrap());
static SEMICOLON_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r";+\n").unwrap());
static NEWLINE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{2,}").unwrap());
static SVG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<svg([^>]*)>").unwrap());

pub fn read_dir_and_sort(path: &str, ext_filter: &[String]) -> Result<Vec<String>, String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Failed to read directory metadata for {}: {}", path, error))?;

    if !metadata.is_dir() {
        return Err(format!("{} is not a directory.", path));
    }

    let mut entries: Vec<String> = Vec::new();
    let normalized = normalize_extensions(ext_filter);

    let read_dir = fs::read_dir(path)
        .map_err(|error| format!("Failed to read directory {}: {}", path, error))?;

    for entry in read_dir.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy().to_string();
        if normalized.is_empty() || has_matching_extension(&name, &normalized) {
            entries.push(name);
        }
    }

    entries.sort();
    Ok(entries)
}

pub fn make_component_name(filename: &str) -> String {
    let sanitized = filename.replace(' ', "");
    sanitized
        .split('-')
        .map(|section| {
            let mut chars = section.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

pub fn component_template(component_name: &str, content: &str) -> Result<String, String> {
    let template = format!(
        "\
import React from \"react\";\n\nfunction {component_name}(props: React.JSX.IntrinsicElements[\"svg\"]) {{\n  return (\n    {content}\n  );\n}}\n\nexport default {component_name};\n"
    );
    format_svg_component(&template)
}

pub fn format_svg_component(content: &str) -> Result<String, String> {
    let has_props = content.contains("{...props}");
    let mut formatted = content.replace("\r\n", "\n");

    formatted = formatted
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    formatted = formatted.replace("\t", "  ");
    formatted = SPACE_RE.replace_all(&formatted, "  ").to_string();
    formatted = SEMICOLON_RE.replace_all(&formatted, ";\n").to_string();
    formatted = NEWLINE_RE.replace_all(&formatted, "\n\n").to_string();

    if !has_props {
        formatted = SVG_RE
            .replacen(&formatted, 1, "<svg$1 {...props}>")
            .to_string();
    }

    let lines: Vec<&str> = formatted.lines().collect();
    let mut indented: Vec<String> = Vec::new();
    let mut indent_level = 0usize;

    for line in lines {
        let trimmed_line = line.trim();

        if trimmed_line.starts_with("</") {
            indent_level = indent_level.saturating_sub(1);
        }

        indented.push(format!("{}{}", "  ".repeat(indent_level), trimmed_line));

        if trimmed_line.starts_with('<')
            && !trimmed_line.starts_with("</")
            && !trimmed_line.contains("/>")
            && !trimmed_line.ends_with("?>")
        {
            indent_level += 1;
        }
    }

    Ok(indented.join("\n").trim().to_string() + "\n")
}

pub fn read_to_string(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))
}

pub fn write_string(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content)
        .map_err(|error| format!("Failed to write {}: {}", path.display(), error))
}

pub fn ensure_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|error| format!("Failed to create directory {}: {}", path.display(), error))
}

pub fn move_file(from: &Path, to: &Path) -> Result<(), String> {
    fs::rename(from, to).map_err(|error| format!("Failed to move {}: {}", from.display(), error))
}

pub fn join_path(base: &str, file: &str) -> PathBuf {
    let mut path = PathBuf::from(base);
    path.push(file);
    path
}

fn normalize_extensions(ext_filter: &[String]) -> Vec<String> {
    ext_filter
        .iter()
        .map(|ext| ext.trim())
        .filter(|ext| !ext.is_empty())
        .map(|ext| {
            if ext.starts_with('.') {
                ext.to_lowercase()
            } else {
                format!(".{}", ext.to_lowercase())
            }
        })
        .collect()
}

fn has_matching_extension(name: &str, extensions: &[String]) -> bool {
    let lower = name.to_lowercase();
    extensions.iter().any(|ext| lower.ends_with(ext))
}

#[cfg(test)]
mod tests {
    use super::{format_svg_component, make_component_name};

    #[test]
    fn makes_component_name_from_hyphenated_filename() {
        assert_eq!(make_component_name("arrow-left icon"), "ArrowLefticon");
        assert_eq!(make_component_name("user-avatar"), "UserAvatar");
    }

    #[test]
    fn format_svg_component_injects_props_and_indents_svg() {
        let formatted = format_svg_component("<svg>\n  <g>\n    <path />\n  </g>\n</svg>\n")
            .expect("svg should format");

        assert_eq!(
            formatted,
            "<svg {...props}>\n  <g>\n    <path />\n  </g>\n</svg>\n"
        );
    }

    #[test]
    fn format_svg_component_does_not_duplicate_props() {
        let formatted = format_svg_component("<svg {...props}>\n<path />\n</svg>\n")
            .expect("svg should format");

        assert_eq!(formatted.matches("{...props}").count(), 1);
    }
}
