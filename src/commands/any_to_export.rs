use std::collections::HashMap;
use std::path::Path;

use crate::cli::args::parse_args;
use crate::utils::fs::{make_component_name, read_dir_and_sort, write_string};

pub fn any_to_export(extensions: Vec<&str>, args: Vec<String>) -> Result<(), String> {
    let parsed = parse_args(&args);
    let directory = parsed.positional.get(0).cloned().unwrap_or_default();
    let output_file = parsed.positional.get(1).cloned().unwrap_or_default();
    let dry_run = parsed.options.get("dry-run").is_some();
    let custom_extensions = parsed.options.get("ext").cloned().unwrap_or_default();

    if parsed.options.get("help").is_some() {
        println!("Usage: -img2export <directory> <output_file> [--ext=.svg,.png] [--dry-run]");
        return Ok(());
    }

    if directory.is_empty() || output_file.is_empty() {
        println!("Missing required arguments. Expected: <directory> <output_file>");
        return Ok(());
    }

    let ext_list: Vec<String> = if !custom_extensions.is_empty() {
        custom_extensions
            .split(',')
            .map(|ext| ext.trim().to_string())
            .collect()
    } else {
        extensions.iter().map(|ext| ext.to_string()).collect()
    };

    let files = read_dir_and_sort(&directory, &ext_list);

    if files.is_empty() {
        println!("No matching files found in {}", directory);
        return Ok(());
    }

    let mut name_counts: HashMap<String, usize> = HashMap::new();
    let export_names: Vec<String> = files
        .iter()
        .map(|file| {
            let filename = file.split('.').next().unwrap_or("");
            let base_name = make_component_name(filename);
            let count = name_counts.get(&base_name).cloned().unwrap_or(0);
            name_counts.insert(base_name.clone(), count + 1);
            if count > 0 {
                format!("{}{}", base_name, count + 1)
            } else {
                base_name
            }
        })
        .collect();

    let output = files
        .iter()
        .zip(export_names.iter())
        .map(|(file, export_name)| {
            format!(
                "export {{ default as {} }} from \"./{}\";",
                export_name, file
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

    let path = Path::new(&directory).join(&output_file);
    if dry_run {
        println!("anyToExport - Dry run. Would write {}", path.display());
        return Ok(());
    }

    write_string(&path, &output)?;
    println!("anyToExport - Done! Wrote {}", path.display());
    Ok(())
}
