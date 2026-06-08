# AGENTS.md

## Project

Rust MCP server for exposing controlled Arch Linux filesystem, Git, Docker, and system tools to Claude Desktop.

## Rules

- Do not modify production code unless explicitly approved.
- Prefer adding tests over refactoring.
- Keep test changes small and reviewable.
- Do not add dependencies without asking first.
- Do not add destructive tools.
- Do not change MCP behavior unless a test proves a bug.
- Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` before finishing.

## Current priority

Generate rich, fast tests for protocol correctness, tool schemas, filesystem safety, and Git command behavior.

## Done means

- Tests compile.
- Tests pass quickly.
- No production behavior changed without approval.
- Summary explains what each test verifies.
