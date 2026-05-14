# JPCG — 剑心PVP计算器

Rust workspace for a 剑网3 damage calculator. Edition 2024 (requires rustc >= 1.85).

## Workspace

| Path | Role |
|------|------|
| `crates/jpcg_core/` | Core calculation engine + FFI exports (`extern "C"`) |
| `crates/jpcg_const/` | Drug/food buff constants |
| `crates/jpcg_update/` | App + data auto-update (fetches from `nefinita-ai.com`) |
| `examples/jpcg_app/src-tauri/` | Tauri v2 desktop app (vanilla HTML/CSS/JS, `@tauri-apps/api` v2) |
| `server_tools/manifest-gen/` | CLI to generate `data_manifest.toml` from `./data/` dir |
| `server_tools/forum/` | Web forum for uploading/downloading `.toml` data files |

## Commands

```sh
cargo build                          # build all workspace members
cargo run -p manifest-gen -- --version v2.0.X --data-dir ./data
# generates data_manifest.toml (SHA256 manifest)

cargo run -p forum                   # start the data file sharing forum (port 8080)
# optional: PORT=9090 FORUM_DATA_DIR=./forum_data cargo run -p forum

# Tauri dev (from examples/jpcg_app/):
# npx tauri dev    (requires @tauri-apps/cli installed)
```

No test, lint, format, or CI infrastructure exists.

## Key facts

- **Chinese-language** code: field names, comments, config keys are all Chinese (pinyin).
- **Data files** are TOML in `data/pvp36500/`. Each profession has one `.toml` (e.g., `mowen.toml`, `zhoutian.toml`). The calculation engine locates these via `std::env::current_exe()` parent.
- **DTO `player.jcsx` is a `String`** — must NOT be coerced to number. Previously broke deserialization (`invalid type: integer 0, expected a string`).
- **Update server**: `https://nefinita-ai.com/updates/JPCG/` (stable) / `JPCG_beta/` (beta). `data_manifest.toml` lives under `https://nefinita-ai.com/files/JPCG/`.
- **Tauri commands** are in `examples/jpcg_app/src-tauri/src/commands/mod.rs`. Frontend JS is vanilla (no framework) in `examples/jpcg_app/src/js/`. Tauri events use `"update-progress"`.
- **FFI**: `jpcg_core` exports `#[no_mangle] extern "C"` functions (`start_calculation`) with `#[repr(C)]` types for cross-language use.
- **No `.unwrap()` / `.expect()` in library code** — all error paths use `Result<_, Error>` to avoid Tauri command hangs (previously observed panic bug in `cal.rs`/`io.rs`).
- **No `rust-toolchain.toml`** — relies on system `rustup default`. CI is absent.

## Agreement: change log

Each modification to the codebase MUST be accompanied by a markdown file under `changes/` describing what was changed, why, and any notable decisions. The file name follows the format `YYYY-MM-DD-HHMMSS.md` (local time).
