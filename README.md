# CozyUtils (Rust Port)

This is a Rust port of the CozyUtils CLI focused on producing a small, self-contained
binary for macOS, Linux, and Windows.

## Prerequisites

- Rust toolchain (stable)

## Build

```bash
cargo build --release
```

The binary will be at `target/release/cozyutils`.

## Size-focused builds

The release profile enables LTO and stripping. If you still need to squeeze size:

```bash
RUSTFLAGS="-C link-arg=-s" cargo build --release
```

On Linux, you can also build a musl static binary:

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

## Cross-platform builds

```bash
# macOS (Intel)
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# macOS (Apple Silicon)
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Windows (MSVC)
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

## Usage

The CLI flags mirror the Bun version.

```bash
./cozyutils -svg2tsx ./icons --dry-run
./cozyutils -img2export ./icons ./index.ts
./cozyutils -prmsg --base=origin/dev --clipboard
```

## Commands

- `-svg2tsx <directory> [--ext=.svg] [--dry-run] [--force] [--no-move]`
- `-img2export <directory> <output_file> [--ext=.svg,.png] [--dry-run]`
- `-prmsg [--base=origin/dev] [--out=path] [--model=gemini-3-flash-preview] [--clipboard] [--clipboard-only] [--copy]`

## Environment

The `-prmsg` command looks for the API key in this order:

1) `GEMINI_API_KEY` environment variable
2) `~/.cozyutils/config.json`

`~/.cozyutils/config.json` example:

```json
{
  "gemini_api_key": "YOUR_KEY_HERE"
}
```

You can create the config with:

```bash
./cozyutils -prmsg --setup --key=YOUR_KEY
```
