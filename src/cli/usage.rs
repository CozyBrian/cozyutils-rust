const COMMANDS: &[(&str, &str, &str)] = &[
    (
        "-cmsg",
        "[--out=path] [--model=MODEL] [--backend=gemini|opencode] [--clipboard] [--clipboard-only] [--commit]",
        "Generate a commit message from staged changes via Gemini or OpenCode",
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
        "[--base=origin/dev] [--out=path] [--model=MODEL] [--backend=gemini|opencode] [--clipboard] [--clipboard-only] [--setup]",
        "Generate a PR message from git diffs via Gemini or OpenCode",
    ),
    (
        "-config",
        "[--show] [--path] [--set-backend=gemini|opencode] [--unset-backend] [--set-key=VALUE] [--unset-key]",
        "Inspect and update ~/.cozyutils/config.json",
    ),
];

pub fn usage() -> String {
    let mut text = String::from("\nUsage:\n");
    for (flag, args, _) in COMMANDS {
        text.push_str(&format!("  {} {}\n", flag, args));
    }
    text.push_str("\n");
    text
}

pub fn help() -> String {
    let mut text = String::from("\nOptions:\n");
    for (flag, args, description) in COMMANDS {
        text.push_str(&format!("  {} {}  {}\n", flag, args, description));
    }
    text.push_str(
    "\nFlags by command:\n  -svg2tsx\n    --ext=.svg          Override extensions to include\n    --dry-run           Print planned changes only\n    --force             Overwrite existing output files\n    --no-move           Keep original SVGs in place\n  -cmsg\n    --out=path           Output commit message to a file\n    --model=MODEL        Override model name\n    --backend=VALUE      Select backend: gemini or opencode\n    --clipboard          Copy commit message to clipboard\n    --clipboard-only     Only copy to clipboard (skip stdout/file)\n    --copy               Copy commit message to clipboard\n    --commit             Run git commit with generated message\n  -config\n    --show               Print config with secrets masked\n    --path               Print config file path\n    --set-backend=VALUE  Set default backend: gemini or opencode\n    --unset-backend      Remove default backend from config\n    --set-key=VALUE      Set Gemini API key in config\n    --unset-key          Remove Gemini API key from config\n  -img2export\n    --ext=.svg,.png     Override extensions to include\n    --dry-run           Print planned changes only\n  -prmsg\n    --base=origin/dev    Base ref for PR message generation\n    --out=path           Output PR message to a file\n    --model=MODEL        Override model name\n    --backend=VALUE      Select backend: gemini or opencode\n    --clipboard          Copy PR message to clipboard\n    --clipboard-only     Only copy to clipboard (skip stdout/file)\n    --copy               Copy PR message to clipboard\n    --setup              Create ~/.cozyutils/config.json\n    --key=VALUE          API key for --setup when using Gemini\n  Global\n    --help, -h          Show help\n    --version, -v       Show version\n\n",
  );
    text
}
