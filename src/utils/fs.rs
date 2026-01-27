use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

pub fn read_dir_and_sort(path: &str, ext_filter: &[String]) -> Vec<String> {
  let metadata = match fs::metadata(path) {
    Ok(value) => value,
    Err(_) => {
      println!("Directory not found: {}", path);
      return Vec::new();
    }
  };

  if !metadata.is_dir() {
    println!("{} is not a directory.", path);
    return Vec::new();
  }

  let mut entries: Vec<String> = Vec::new();
  let normalized = normalize_extensions(ext_filter);

  let read_dir = match fs::read_dir(path) {
    Ok(value) => value,
    Err(_) => {
      println!("Directory not found: {}", path);
      return Vec::new();
    }
  };

  for entry in read_dir.flatten() {
    let file_name = entry.file_name();
    let name = file_name.to_string_lossy().to_string();
    if normalized.is_empty() || has_matching_extension(&name, &normalized) {
      entries.push(name);
    }
  }

  entries.sort();
  entries
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

pub fn component_template(component_name: &str, content: &str) -> String {
  let template = format!(
    "\
import React from \"react\";\n\nfunction {component_name}(props: React.JSX.IntrinsicElements[\"svg\"]) {{\n  return (\n    {content}\n  );\n}}\n\nexport default {component_name};\n"
  );
  format_svg_component(&template)
}

pub fn format_svg_component(content: &str) -> String {
  let has_props = content.contains("{...props}");
  let mut formatted = content.replace("\r\n", "\n");
  let space_re = Regex::new(r" {2,}").unwrap();
  let semicolon_re = Regex::new(r";+\n").unwrap();
  let newline_re = Regex::new(r"\n{2,}").unwrap();
  let svg_re = Regex::new(r"<svg([^>]*)>").unwrap();

  formatted = formatted
    .lines()
    .filter(|line| !line.trim().is_empty())
    .map(|line| line.trim_end())
    .collect::<Vec<_>>()
    .join("\n");

  formatted = formatted.replace("{...props}", "{...props}");
  formatted = formatted.replace("\t", "  ");
  formatted = space_re.replace_all(&formatted, "  ").to_string();
  formatted = semicolon_re.replace_all(&formatted, ";\n").to_string();
  formatted = newline_re.replace_all(&formatted, "\n\n").to_string();

  if !has_props {
    formatted = svg_re
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

    if trimmed_line.starts_with('<') && !trimmed_line.starts_with("</") {
      if !trimmed_line.contains("/>") && !trimmed_line.ends_with("?>") {
        indent_level += 1;
      }
    }
  }

  indented.join("\n").trim().to_string() + "\n"
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
  fs::rename(from, to)
    .map_err(|error| format!("Failed to move {}: {}", from.display(), error))
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
