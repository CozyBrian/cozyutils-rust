use std::path::Path;

use regex::Regex;

use crate::cli::args::parse_args;
use crate::commands::any_to_export::any_to_export;
use crate::utils::fs::{
    component_template, ensure_dir, join_path, make_component_name, move_file, read_dir_and_sort,
    read_to_string, write_string,
};

pub fn svg_to_tsx(args: Vec<String>) -> Result<(), String> {
    let parsed = parse_args(&args);
    let directory = parsed.positional.get(0).cloned().unwrap_or_default();
    let dry_run = parsed.options.get("dry-run").is_some();
    let force = parsed.options.get("force").is_some();
    let no_move = parsed.options.get("no-move").is_some();
    let custom_extensions = parsed.options.get("ext").cloned().unwrap_or_default();

    if parsed.options.get("help").is_some() {
        println!("Usage: -svg2tsx <directory> [--ext=.svg] [--dry-run] [--force] [--no-move]");
        return Ok(());
    }

    if directory.is_empty() {
        println!("Missing required argument. Expected: <directory>");
        return Ok(());
    }

    let ext_list: Vec<String> = if !custom_extensions.is_empty() {
        custom_extensions
            .split(',')
            .map(|ext| ext.trim().to_string())
            .collect()
    } else {
        vec![".svg".to_string()]
    };

    let files = read_dir_and_sort(&directory, &ext_list);

    if files.is_empty() {
        println!("No matching files found in {}", directory);
        return Ok(());
    }

    let dashed_attribute_regex =
        Regex::new(r"(\w+)-(\w+)").map_err(|error| format!("Invalid regex: {}", error))?;
    let fill_regex = Regex::new(r###"fill="(?!none)([^"\s]+)""###)
        .map_err(|error| format!("Invalid regex: {}", error))?;
    let stroke_hex_regex = Regex::new(r###"stroke="#([^"\s]+)""###)
        .map_err(|error| format!("Invalid regex: {}", error))?;
    let stroke_regex = Regex::new(r###"stroke="([^"\s]+)""###)
        .map_err(|error| format!("Invalid regex: {}", error))?;

    for filename in &files {
        let path = join_path(&directory, filename);
        let mut content = read_to_string(&path)?;

        let filename_no_ext = filename.split('.').next().unwrap_or("");
        let component_name = make_component_name(filename_no_ext);

        content = dashed_attribute_regex
            .replace_all(&content, |captures: &regex::Captures| {
                let first = captures.get(1).map(|value| value.as_str()).unwrap_or("");
                let second = captures.get(2).map(|value| value.as_str()).unwrap_or("");
                let mut uppercase = second.chars();
                match uppercase.next() {
                    Some(ch) => format!("{}{}{}", first, ch.to_uppercase(), uppercase.as_str()),
                    None => format!("{}", first),
                }
            })
            .to_string();

        content = fill_regex
            .replace_all(&content, "fill=\"currentColor\"")
            .to_string();
        content = stroke_hex_regex
            .replace_all(&content, "stroke=\"currentColor\"")
            .to_string();
        content = stroke_regex
            .replace_all(&content, "stroke=\"currentColor\"")
            .to_string();
        content = content.replace("class=\"", "className=\"");
        content = content.replace("clip-rule=\"", "clipRule=\"");
        content = content.replace("fill-rule=\"", "fillRule=\"");
        content = content.replace("stroke-linecap=\"", "strokeLinecap=\"");
        content = content.replace("stroke-linejoin=\"", "strokeLinejoin=\"");
        content = content.replace("stroke-width=\"", "strokeWidth=\"");

        let component_content = component_template(&component_name, &content);
        let output_path = Path::new(&directory).join(format!("{}.tsx", component_name));

        if output_path.exists() && !force {
            println!("File {}.tsx already exists. Skipping...", component_name);
            continue;
        }

        if dry_run {
            println!("svgToTsx - Dry run. Would write {}", output_path.display());
            continue;
        }

        write_string(&output_path, &component_content)?;

        if !no_move {
            let new_svg_path = Path::new(&directory).join("original").join(filename);
            let old_svg_path = Path::new(&directory).join(filename);
            ensure_dir(new_svg_path.parent().unwrap_or(Path::new(&directory)))?;
            move_file(&old_svg_path, &new_svg_path)?;
        }
    }

    if !dry_run {
        any_to_export(
            vec![".tsx"],
            vec![directory.clone(), "index.ts".to_string()],
        )?;
    }

    println!("svgToTsx - Done! Processed {} file(s).", files.len());
    Ok(())
}
