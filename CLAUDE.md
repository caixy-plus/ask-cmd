# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`ask` is a Rust CLI that converts natural-language requests into shell commands via AI providers (Claude Code CLI, OpenCode, Codex, Cursor). It does **not** call AI APIs directly — it shells out to locally-installed AI CLI tools.

## Commands

```bash
cargo build                  # build debug
cargo build --release        # build release (binary: target/release/ask)
cargo test                   # run all tests
cargo test suggestions       # run tests in a specific module
cargo clippy -- -D warnings  # lint
cargo install --path .       # install locally as `ask`
```

## Architecture

**Provider resolution** (`src/providers/registry.rs`): picks a provider in priority order — CLI flag → `ASK_PROVIDER` env var → `~/.config/ask-cmd/config.json` → first detected installed provider. Each provider implements the `Provider` trait (`src/providers/provider.rs`) with `suggest_sync` (returns JSON array of commands) and `query_sync` (returns single command).

**Suggestion parsing** (`src/suggestions.rs`): `parse_suggestions` strips markdown fences, parses a JSON array from the AI response, filters out non-commands (CJK text, empty strings), deduplicates, and caps at 5. The `"no JSON array in response"` error means the AI returned prose instead of a `["cmd1", "cmd2", ...]` array — check the prompt in `src/prompt.rs`.

**Validation** (`src/validate.rs`): `looks_like_command` rejects strings starting with non-ASCII or containing CJK characters. `is_dangerous` regex-matches patterns like `sudo`, `rm -rf`, `dd`, `mkfs` and triggers a warning before execution.

**Interactive picker** (`src/ui/picker.rs`): renders a terminal picker using `crossterm` in the alternate screen buffer. Returns `PickResult::Selected(idx)`, `Retry`, or `Cancel`.

**Config** (`src/config.rs`): persists the default provider choice to `~/.config/ask-cmd/config.json` as `{"default_provider": "claude"}`.

**Shell integration** (`src/shell/`): `ask install` writes a PATH export to the user's shell rc. `shell::purge_legacy_integration` removes old wrapper functions from prior versions.

## Adding a Provider

1. Create `src/providers/<name>.rs` implementing the `Provider` trait.
2. Register it in `src/providers/registry.rs` → `all_providers()`.
3. Re-export availability/hint functions in `src/providers/mod.rs` and `src/lib.rs`.

The provider's `suggest_sync` must return output that `parse_suggestions` can handle: a JSON array of command strings, optionally wrapped in a markdown fence.
