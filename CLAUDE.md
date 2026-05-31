# CLAUDE.md

Guidance for Claude Code (and humans) working in this repository.

## What this is

`lancer-aderyn` is a Rust reimplementation of [Cyfrin Aderyn](https://github.com/Cyfrin/aderyn) — a fast Solidity static analyzer — extended with a native desktop GUI built on egui. It is part of the Lancer ecosystem, a 2026 AI-era bug-bounty toolkit. The tool traverses the Solidity AST emitted by `solc`, runs 88 detectors (39 high-severity + 49 low-severity), and emits Markdown/JSON/SARIF reports, while also shipping a built-in LSP server and MCP server from a single static binary.

## Architecture

- `src/lib.rs` — library entry point; exports the public API consumed by `main.rs` and tests
- `src/main.rs` — binary entry point; wires CLI args to the driver
- `src/cli.rs` — CLI surface (clap-based flag and subcommand definitions)
- `src/ast/` — Solidity AST parsing and traversal logic
- `src/detect/` — detector implementations (88 detectors: high- and low-severity)
- `src/driver/` — orchestration layer that runs the full analysis pipeline
- `src/context/` — analysis context and shared state passed through the pipeline
- `src/report/` — report generation (Markdown, JSON, SARIF output formats)
- `src/gui/` — egui desktop GUI (feature-gated behind the `gui` feature flag)
- `src/lsp/` — LSP server for editor diagnostics (stdio transport)
- `src/mcp/` — MCP server so Claude/Cursor can invoke the analyzer as a tool
- `src/audit/` — auditor-mode scaffold generation
- `src/config/` — `aderyn.toml` parsing and configuration resolution
- `src/self_test.rs` — self-test harness for the analysis engine
- `tests/` — integration tests

## Conventions

- Error propagation uses `eyre::Result`; avoid `unwrap`/`expect` on fallible I/O.
- GUI is gated behind the default `gui` feature; `--no-default-features` builds a headless CLI binary.
- Library code lives in `src/lib.rs`; binary entry point in `src/main.rs`.
- Parallelism uses `rayon`; keep detector implementations `Send + Sync`.
- Report format is selected by output file extension (`.md`, `.json`, `.sarif`); format logic lives in `src/report/`.
- Source-level ignore directives (`// aderyn-ignore`, `// aderyn-fp`) are parsed during AST traversal, not at the detector level.

## Build / test / run

```
build.bat                   # Windows → build\lancer-aderyn.exe
./build-linux.sh            # Linux
./build-mac.sh              # macOS
cargo build --release       # standard release build
cargo test                  # unit + integration tests
```

## Repo notes

- `plan.md` is local-only and must NOT be committed. It is gitignored; if ever tracked, run `git rm --cached plan.md`.
- `.claude/` and `target/`/`build/`/`dist/` are local-only and gitignored.
