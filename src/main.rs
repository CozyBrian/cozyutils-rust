mod cli;
mod commands;
mod utils;

use cli::usage::{help, usage};

fn main() {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let rest: Vec<String> = args.collect();

    if matches!(command.as_str(), "-v" | "--version") {
        let name = env!("CARGO_PKG_NAME");
        let version = env!("CARGO_PKG_VERSION");
        println!("{} {} (rust)", name, version);
        return;
    }

    if command.is_empty() || !command.starts_with('-') {
        print!("{}", usage());
        std::process::exit(1);
    }

    if matches!(command.as_str(), "-help" | "-h" | "--help") {
        print!("{}", help());
        return;
    }

    let result = match command.as_str() {
        "-svg2tsx" => commands::svg_to_tsx::svg_to_tsx(rest),
        "-img2export" => commands::any_to_export::any_to_export(
            vec![".svg", ".jpg", ".jpeg", ".png", ".gif", ".webp", ".tsx"],
            rest,
        ),
        "-cmsg" => commands::commit_message::commit_message(rest),
        "-config" => commands::config::config_command(rest),
        "-prmsg" => commands::pr_message::pr_message(rest),
        _ => {
            eprintln!("Invalid command: {}", command);
            eprint!("{}", usage());
            std::process::exit(1);
        }
    };

    if let Err(error) = result {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
