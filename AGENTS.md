# JPCG — 剑心PVP计算器

Rust workspace for a 剑网3 damage calculator. Edition 2024 (requires rustc >= 1.85).

## Workspace

| Path | Role |
|------|------|
| `crates/jpcg_core/` | Core calculation engine + FFI exports (`extern "C"`) |
| `crates/jpcg_const/` | Drug/food buff constants (unused by other crates) |
| `crates/jpcg_update/` | App + data auto-update (fetches from `nefinita-ai.com`) |
| `examples/jpcg_app/src-tauri/` | Tauri v2 desktop app (this IS the app, not a mere example) |
| `server_tools/manifest-gen/` | CLI to generate `data_manifest.toml` from `./data/` |
| `server_tools/forum/` | Web forum for uploading/downloading `.toml` data files |

## Commands

```sh
cargo build                          # build all workspace members (incl. Tauri app)
cargo run -p manifest-gen -- --version v2.0.X --data-dir ./data
# also accepts: --output data_manifest.toml (default)

cargo run -p forum                   # start forum (port 8080 by default)
# PORT=9090 FORUM_DATA_DIR=./forum_data cargo run -p forum

# Tauri dev (from examples/jpcg_app/):
# npx tauri dev    (requires @tauri-apps/cli, frontendDist: ../src)
```

No CI, test suites, or formatter config exist.

## Key facts

- **Chinese-language code**: field names, comments, config keys are Chinese (pinyin).
- **Data files** are TOML in `data/pvp36500/`. Each profession has one `.toml` (e.g., `mowen.toml`). Core engine locates these via `std::env::current_exe()` parent → `data/pvp36500/{xinfa}.toml`.
- **`player.jcsx` is a `String`** — must NOT be coerced to number. Previously broke deserialization.
- **`saved_config.toml`** reads/writes from CWD (e.g., `examples/jpcg_app/src-tauri/`), NOT exe parent dir.
- **`toml_input()` appends `.toml`** — callers pass the path without the extension.
- **Update server**: `https://nefinita-ai.com/updates/JPCG/` (stable) / `JPCG_beta/` (beta). `data_manifest.toml` at `https://nefinita-ai.com/files/JPCG/`.
- **Tauri commands** are in `examples/jpcg_app/src-tauri/src/commands/mod.rs`. Frontend JS is vanilla (ES modules, no bundler) in `examples/jpcg_app/src/js/`. Tauri events use `"update-progress"`.
- **FFI** (`jpcg_core`): `#[no_mangle] extern "C"` function `start_calculation` with `#[repr(C)]` types. For cross-language use only; Tauri app uses Rust API directly.
- **Tauri lib name**: `jpcg_app_lib` (not `jpcg_app`) due to Windows crate-type conflict.
- **No `.unwrap()` / `.expect()`** in library code — all errors use `Result<_, Error>` or safe defaults. Previously caused Tauri command hangs.
- **No `rust-toolchain.toml`** — relies on `rustup default`.

## Agreement: change log

Each modification MUST be accompanied by a markdown file under `changes/` named `YYYY-MM-DD-HHMMSS.md` describing what, why, and notable decisions.
