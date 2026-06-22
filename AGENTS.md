# AGENTS.md

## Commands

- Use `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, then `cargo test` for full validation.
- Use `cargo run` to inspect the full default fetch output.
- Use `cargo run -- cpu memory` or another module list to inspect compact explicit output.
- Use `cargo run -- list` to verify module names exposed to users.

## CLI Semantics

- `omafetch` renders the full default module order from `ModuleRegistry::defaults()`.
- `omafetch <modules...>` renders compact identity context first, then requested modules in request order; compact context is `omarchy theme host os kernel wm`.
- `omafetch list` prints registry module names.
- Unknown modules should fail with the unknown name plus the available module list.

## Architecture

- Entry flow is `src/main.rs` -> `src/app.rs` -> `ModuleRegistry` -> module `collect()` -> `RenderDocument` -> `render_document()`.
- Add or reorder modules through `src/modules/registry.rs`; it owns typed `ModuleKey`, available names, full defaults, and compact defaults.
- A module usually needs three edits: add `src/modules/<name>.rs`, export it in `src/modules/mod.rs`, and register it in `ModuleRegistry` with a `ModuleKey`.
- Keep probes in `src/probe/`; modules should format facts, not scatter filesystem/env/command probing logic.
- Rendering grouping belongs in `src/render/document.rs`; terminal layout, truncation width, logo offset, and ANSI styling belong in `src/render/layout.rs` / `src/render/text.rs`.

## Omarchy Sources

- Omarchy state comes from existing Omarchy files; do not add an omafetch config/theme system.
- Current Omarchy paths are centralized in `src/omarchy/paths.rs`.
- Theme discovery reads `~/.config/omarchy/current/theme.name` and `~/.config/omarchy/current/theme/colors.toml`.
- Omarchy version reads `~/.local/share/omarchy/version`; do not use the omafetch crate version for the `Omarchy` row.

## Probe Constraints

- Do not shell out to fastfetch, neofetch, screenfetch, or similar.
- If a command is necessary, use `probe::command::run_capture(program, args)`; never build shell command strings.
- Missing host data should render `unknown` or be skipped only by deliberate app-level default filters, as with unavailable `battery` and `gtt-memory`.
- Tests should use fixture strings for parsers; avoid tests depending on this machine's hardware.

## Output Rules

- Buck art is intentionally ANSI-color cascaded using terminal theme colors.
- Labels are dim/muted; values are normal foreground.
- Values are single-line and terminal-width aware; content-aware shortening lives in `render/text.rs`.
- Default output may include group gaps; explicit module output should stay compact.

## Packaging

- Starter Arch packaging is in `packaging/PKGBUILD`.
- Package installs the binary to `/usr/bin/omafetch`, README to `/usr/share/doc/omafetch/`, and MIT license to `/usr/share/licenses/omafetch/`.
