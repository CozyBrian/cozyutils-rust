const COMMANDS: &[(&str, &str, &str)] = &[
    (
        "-cmsg",
        "[--out=path] [--model=MODEL] [--provider=NAME] [--backend=NAME] [--clipboard] [--clipboard-only] [--commit]",
        "Generate a commit message from staged changes via a configured AI provider",
    ),
    (
        "-svg2tsx",
        "<directory> [--ext=.svg] [--dry-run] [--force] [--no-move]",
        "Convert SVG files in a directory to React components",
    ),
    (
        "-img2export",
        "<directory> <output_file> [--ext=.svg,.png] [--dry-run]",
        "Export image files in a directory as named exports",
    ),
    (
        "-prmsg",
        "[--base=origin/dev] [--out=path] [--model=MODEL] [--provider=NAME] [--backend=NAME] [--clipboard] [--clipboard-only]",
        "Generate a PR message from git diffs via a configured AI provider",
    ),
    (
        "-config",
        "[--init] [--force] [--show] [--path] [--set-default-provider=NAME] [--set-command-provider=COMMAND:NAME] ...",
        "Inspect and update ~/.cozyutils/config.json",
    ),
];

pub fn usage() -> String {
    let mut text = String::from("\nUsage:\n");
    for (flag, args, _) in COMMANDS {
        text.push_str(&format!("  {} {}\n", flag, args));
    }
    text.push('\n');
    text
}

pub fn help() -> String {
    let mut text = String::from("\nOptions:\n");
    for (flag, args, description) in COMMANDS {
        text.push_str(&format!("  {} {}  {}\n", flag, args, description));
    }
    text.push_str(
    "\nFlags by command:\n  -svg2tsx\n    --ext=.svg                Override extensions to include\n    --dry-run                 Print planned changes only\n    --force                   Overwrite existing output files\n    --no-move                 Keep original SVGs in place\n  -cmsg\n    --out=path                Output commit message to a file\n    --model=MODEL             Override model name\n    --provider=NAME           Select configured provider\n    --backend=NAME            Alias for --provider\n    --clipboard               Copy commit message to clipboard\n    --clipboard-only          Only copy to clipboard (skip stdout/file)\n    --copy                    Copy commit message to clipboard\n    --commit                  Run git commit with generated message\n  -config\n    --init                    Write a starter config with built-in providers\n    --force                   Overwrite config when used with --init\n    --show                    Print config with secrets masked\n    --path                    Print config file path\n    --set-default-provider=NAME      Set the global default provider\n    --unset-default-provider         Remove the global default provider\n    --set-command-provider=CMD:NAME  Set provider for prmsg or cmsg\n    --unset-command-provider=CMD     Remove provider override for prmsg or cmsg\n    --set-command-model=CMD:MODEL    Set model override for prmsg or cmsg\n    --unset-command-model=CMD        Remove model override for prmsg or cmsg\n    --set-provider-type=NAME:TYPE    Set provider type: gemini, openai-compatible, opencode\n    --set-provider-base-url=NAME:URL Set base URL for an openai-compatible provider\n    --set-provider-key=NAME:VALUE    Store provider API key in config\n    --unset-provider-key=NAME        Remove stored provider API key\n    --set-provider-key-env=NAME:ENV  Set env var name used for provider API key\n    --unset-provider-key-env=NAME    Remove env var name for provider API key\n    --set-provider-model=NAME:MODEL  Set provider default model\n    --unset-provider-model=NAME      Remove provider default model\n  -img2export\n    --ext=.svg,.png           Override extensions to include\n    --dry-run                 Print planned changes only\n  -prmsg\n    --base=origin/dev         Base ref for PR message generation\n    --out=path                Output PR message to a file\n    --model=MODEL             Override model name\n    --provider=NAME           Select configured provider\n    --backend=NAME            Alias for --provider\n    --clipboard               Copy PR message to clipboard\n    --clipboard-only          Only copy to clipboard (skip stdout/file)\n    --copy                    Copy PR message to clipboard\n  Global\n    --help, -h                Show help\n    --version, -v             Show version\n\n",
  );
    text
}
