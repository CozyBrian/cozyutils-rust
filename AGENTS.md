# AGENTS.md

This file guides agentic contributors working in this repository.

Repository
- Rust CLI (edition 2024) that mirrors CozyUtils behavior.
- Minimal dependencies, small binary focus.

Build, Lint, Test
- Build (debug): `cargo build`
- Build (release): `cargo build --release`
- Run binary: `./target/debug/cozyutils` or `./target/release/cozyutils`
- Lint (Clippy): `cargo clippy --all-targets --all-features`
- Format (Rustfmt): `cargo fmt`
- Test (all): `cargo test`
- Test (single test by name): `cargo test test_name`
- Test (single module): `cargo test module_path::tests::test_name`
- Test (single file via filter): `cargo test path_or_module_prefix`
- Test (doc tests only): `cargo test --doc`

Release/size builds
- Release with extra stripping: `RUSTFLAGS="-C link-arg=-s" cargo build --release`
- Linux musl build: `rustup target add x86_64-unknown-linux-musl` then
  `cargo build --release --target x86_64-unknown-linux-musl`

Cursor/Copilot rules
- None found in `.cursor/rules/`, `.cursorrules`, or `.github/copilot-instructions.md`.

Project layout
- `src/main.rs` wires CLI commands.
- `src/cli/*` contains argument parsing and usage output.
- `src/commands/*` contains command implementations.
- `src/utils/*` contains filesystem helpers and formatting.

Code style guidelines

Formatting
- Match existing style: two-space indentation in Rust files.
- Keep lines reasonably short; break long argument lists across lines.
- Use trailing commas in multi-line lists.
- Prefer explicit blocks over deeply chained expressions when readability suffers.

Imports
- Group imports by origin: standard library, external crates, then internal `crate::`.
- Keep import blocks compact; avoid unused imports.
- Prefer `use std::path::{Path, PathBuf};` for related items.

Types and ownership
- Prefer `Result<T, String>` for error propagation in commands and utils.
- Use `Option<T>` for optional inputs (e.g., flags, config values).
- Favor borrowing (`&str`, `&Path`) over cloning; clone only when needed.
- Use `Vec<String>` for argument lists and file collections.

Naming conventions
- Rust standard: `snake_case` for functions/vars, `CamelCase` for types.
- CLI flags mirror Bun version; do not rename flags without updating usage strings.
- For generated components, use `make_component_name` to keep naming consistent.

Error handling
- Avoid panics; return `Err(String)` with clear, user-facing messages.
- Use `map_err(|error| format!(...))` to add context.
- Prefer early returns on invalid input (missing args, empty directory).
- Log command execution with clear labels (see `pr_message.rs`).

Logging and user output
- Use `println!` for user-facing output; prefer consistent prefixes
  (e.g., `svgToTsx - ...`, `anyToExport - ...`, `prMessage - ...`).
- When a command supports `--dry-run`, keep output explicit about skipped writes.

CLI parsing
- Flags are parsed by `parse_args` and stored in `ParsedArgs`.
- Boolean flags are detected by name; adding a new boolean flag requires
  updating `is_boolean_flag` in `src/cli/args.rs`.
- Support both `--key=value` and `--key value` forms for non-boolean options.

File system behavior
- Use helpers from `src/utils/fs.rs` for read/write/move to keep
  consistent error messages.
- Always check for missing directories and empty results before writing.
- When writing files, prefer deterministic ordering (`read_dir_and_sort`).

Regex usage
- Compile regex with clear error messages; avoid unwrap.
- Keep regex patterns local to the command that uses them.

Config and environment
- `-prmsg` reads `GEMINI_API_KEY` or `~/.cozyutils/config.json`.
- Do not log secrets; never print API keys or config contents.
- Avoid committing secrets; `.env` exists locally but should not be added to git.

Platform behavior
- Clipboard support uses OS-specific commands (`pbcopy`, `wl-copy`, etc.).
- Keep OS checks graceful; fail with actionable errors when missing.

Tests
- If you add tests, keep them close to the code (module-level `mod tests`).
- Use descriptive test names and cover common failure cases.

Changes and patches
- Avoid refactors unless needed by the requested change.
- Keep functions small and focused; prefer helpers in `src/utils` if reused.
- If you add a new command, update usage/help in `src/cli/usage.rs`.

Commit hygiene (for humans/agents)
- Do not commit generated binaries or files under `target/`.
- Keep diffs scoped to the requested behavior.

Notes on style in this repo
- Two-space indent is used even though rustfmt defaults to 4; preserve existing
  indentation unless a formatter is introduced.
- Most functions return `Result<(), String>` and print errors in `main`.

Useful commands for local validation
- `cargo fmt` then `cargo clippy --all-targets --all-features`
- `cargo test` (or a single test filter as above)

## Codebase Index

This project has a living `docs/` folder with architecture, implementation, patterns, decisions, and changelog files.

### Session Start
- Read `docs/architecture.md` and `docs/implementation.md` before doing any work.
- These files contain the project map — do not re-scan the codebase from scratch.

### ⚠️ Red Flags — Don't Do These

Before searching the codebase, ask: **"Is this information in the docs already?"**

DON'T:
- Use Glob/Grep to explore or understand project structure
- Run broad file searches to learn how things work
- Search the codebase when a doc exists that explains it
- Use codebase-indexer skill unless maintaining docs after changes

DO:
- Start every session reading `docs/architecture.md` and `docs/implementation.md`
- Use targeted Glob/Grep ONLY to find specific files (e.g., you know the filename pattern)
- Check the docs first before jumping into code exploration
- Update docs after every feature/bugfix so future sessions can use them

### Subagents
Subagents spawned via the Agent tool (Explore, Plan, general-purpose) do NOT inherit CLAUDE.md — they start blank. When writing any subagent prompt, include this line at the top:
> "This project has indexed docs. Read `docs/architecture.md` and `docs/implementation.md` first — they are the project map. Do not explore source files for information already covered there."

### After Every Feature or Bugfix
1. Run `git diff HEAD~1 --name-only` to identify changed files.
2. Re-scan only the changed files and their direct neighbors (same package/directory).
3. Update the relevant doc files using targeted edits (do not rewrite unaffected sections):
   - New module/package → update `docs/architecture.md`, `docs/implementation.md`
   - New class/function/endpoint → update `docs/implementation.md`
   - Renamed files/folders → update `docs/architecture.md`, `docs/patterns.md`
   - New dependency → update `docs/architecture.md`
   - New naming/code pattern → update `docs/patterns.md`
4. Ask: "Did this change involve making or reversing an architectural decision?"
   - If yes → append an ADR entry to `docs/decisions.md`
   - If no → skip
5. Append a dated changelog entry to `docs/changelog.md`:
   ```
   ## YYYY-MM-DD — [brief description]
   - What changed
   - Which modules were affected
   ```
