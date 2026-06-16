# CozyUtils (Rust Port)

This is a Rust port of the CozyUtils CLI focused on producing a small, self-contained
binary for macOS, Linux, and Windows.

## Prerequisites

- Rust toolchain (stable)

## Build & Install

To build and install `cozyutils` to `~/.cozyutils` and add it to your `PATH` automatically:

```bash
./install.sh
```

Alternatively, for a manual build:

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
- `-prmsg [--base=origin/dev] [--out=path] [--model=MODEL] [--provider=NAME] [--backend=NAME] [--clipboard] [--clipboard-only] [--copy]`
- `-cmsg [--out=path] [--model=MODEL] [--provider=NAME] [--backend=NAME] [--clipboard] [--clipboard-only] [--commit]`
- `-config [--init] [--force] [--show] [--path] [--set-default-provider=NAME] [--set-command-provider=COMMAND:NAME] [--set-provider-type=NAME:TYPE] ...`

## Config

AI commands now use named providers. `--provider` is the primary flag and `--backend` is kept as an alias.

Built-in provider templates:

- `gemini`
- `openrouter`
- `deepseek`
- `opencode`

You can also define custom `openai-compatible` providers.

Provider API keys are resolved in this order:

1. The provider's configured `api_key_env` environment variable
2. The provider's stored `api_key` in `~/.cozyutils/config.json`

`~/.cozyutils/config.json` example:

```json
{
  "defaults": {
    "provider": "openrouter",
    "commands": {
      "prmsg": {
        "provider": "openrouter",
        "model": "openai/gpt-5.4-mini"
      },
      "cmsg": {
        "provider": "deepseek",
        "model": "deepseek-chat"
      }
    }
  },
  "providers": {
    "openrouter": {
      "type": "openai-compatible",
      "base_url": "https://openrouter.ai/api/v1",
      "api_key_env": "OPENROUTER_API_KEY",
      "default_model": "openai/gpt-5.4-mini"
    },
    "deepseek": {
      "type": "openai-compatible",
      "base_url": "https://api.deepseek.com/v1",
      "api_key_env": "DEEPSEEK_API_KEY",
      "default_model": "deepseek-chat"
    },
    "gemini": {
      "type": "gemini",
      "api_key_env": "GEMINI_API_KEY",
      "default_model": "gemini-3-flash-preview"
    },
    "opencode": {
      "type": "opencode",
      "default_model": "openai/gpt-5.4-mini"
    }
  }
}
```

This is a breaking config change. Older `gemini_api_key` and `backend` config fields are no longer supported.

You can create a starter config with:

```bash
./cozyutils -config --init
```

If the config already exists, overwrite it with:

```bash
./cozyutils -config --init --force
```

You can then update the config with:

```bash
./cozyutils -config --set-default-provider=openrouter
./cozyutils -config --set-command-provider=cmsg:deepseek
./cozyutils -config --set-provider-key-env=openrouter:OPENROUTER_API_KEY
./cozyutils -config --set-provider-key-env=deepseek:DEEPSEEK_API_KEY
./cozyutils -config --show
```

To add a custom OpenAI-compatible provider:

```bash
./cozyutils -config --set-provider-type=myproxy:openai-compatible
./cozyutils -config --set-provider-base-url=myproxy:https://myproxy.example.com/v1
./cozyutils -config --set-provider-key-env=myproxy:MYPROXY_API_KEY
./cozyutils -config --set-provider-model=myproxy:gpt-4.1-mini
./cozyutils -config --set-default-provider=myproxy
./cozyutils -config --show
```
