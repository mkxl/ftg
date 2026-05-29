# AGENTS.md

## Commands
- Use the pinned nightly toolchain from `rust-toolchain.toml` (`nightly-2026-02-11`, edition 2024); do not assume stable Rust works.
- Fast verification: `cargo fmt --check`, `cargo clippy --all-targets --all-features`, then `cargo test`.
- There are currently no unit/integration tests; `cargo test` only builds and runs the empty binary test harness.
- Optional Nix entrypoint: `nix develop` provides the pinned Rust toolchain plus `tokio-console`; `nix build` builds the crate via crane.

## Architecture
- This is a single binary crate, not a workspace. `src/main.rs` only parses `CliArgs` and dispatches.
- Default CLI behavior is connect mode: no subcommand calls `Client::connect`; `ftg serve` runs only the server.
- The client starts a server automatically if `/tmp/ftg.socket` is not accepting connections, waits 25 ms, then connects.
- The server always binds `/tmp/ftg.socket` and removes an existing socket path before binding.
- Runtime flow is `Server` -> `WindowMap`/`BufferMap` -> `Window` -> `ViewStateSet`/`View`; terminal redraw happens on a 15 ms tick in `Server::step`.

## Config And Commands
- The built-in config is `src/default-config.yaml`, embedded with `include_str!`; update it with any config schema/key binding changes.
- User config is loaded with `--config-file` and must deserialize into `Config` in `src/config.rs`.
- Theme config includes `side_tab_percentage`; color config includes `side_tab`, `side_tab_path`, `side_tab_active`, and `side_tab_path_active` for tab bar styling.
- Editor commands are serde-tagged as snake_case in `src/command.rs`; command enum changes must match YAML `binding.command` values.
- Tracing is JSON when `--tracing-file` is supplied; `--tokio-console` is supported and the Nix dev shell includes `tokio-console`.

## Style Guide
- Follow the existing `mkutils::Utils` extension style for small conversions and control flow (`.ok()`, `.some()`, `.output_ok()`, `.check_present()?`, `.pipe_into(...)`) instead of spelling out obvious `Ok(...)`/`Some(...)` wrappers.
- Keep module-local behavior in inherent `impl` blocks with small private helpers and `const` names for fixed labels, durations, and defaults.
- New collection/state holder types commonly expose `empty()` constructors; use `derive_more::Constructor` when the code needs a field-wise `new` constructor.
- Prefer `const fn` for trivial accessors when possible; existing accessors return references directly rather than adding extra abstraction.
- Match local import aliases such as `anyhow::Error as AnyhowError` and `std::io::Error as IoError`; avoid introducing inconsistent `Result` type aliases.
- Existing code uses early-return `let ... else` for absent optional state and logs/drops recoverable per-window/per-file errors instead of crashing the server loop.

## Agent Maintenance
- When a source change alters commands, architecture, config behavior, style conventions, or workflow constraints documented here, update `AGENTS.md` in the same change.

## Code Constraints
- `Cargo.toml` denies Rust warnings and denies Clippy `all`, `cargo`, `nursery`, and `pedantic`; expect new warnings to fail `cargo clippy`.
- Formatting is customized in `rustfmt.toml` (`max_width = 120`, crate-level import grouping), so run rustfmt instead of hand-wrapping to defaults.
- `crossterm` is patched to a specific git revision in `Cargo.toml`; avoid changing the dependency source unless that is the task.
