# lancer-aderyn

A Rust reimplementation of [Cyfrin Aderyn](https://github.com/Cyfrin/aderyn) — a fast
Solidity static analyzer — with a native desktop GUI on top. Part of the Lancer
ecosystem (a 2026 AI-era bug-bounty toolkit).

`lancer-aderyn` traverses the Solidity AST emitted by `solc`, runs **88 detectors**
(39 high-severity + 49 low-severity), and emits a Markdown / JSON / SARIF report of
high- and low-severity findings. It ships a built-in **LSP server** (editor
diagnostics), an **MCP server** (so Claude / Cursor can call it as a tool), and a
pure-Rust **egui GUI** — all from a single static binary.

## Usage

```text
lancer-aderyn                 # open the native GUI (no args)
lancer-aderyn --cli .         # analyze the current project, write report.md
lancer-aderyn ./src --stdout  # print the report to stdout
```

### Common flags

| Flag | Meaning |
|------|---------|
| `-s, --src <DIR>` | source contracts directory (auto-read from `foundry.toml`) |
| `-i, --path-includes <CSV>` | comma-delimited path fragments to include |
| `-x, --path-excludes <CSV>` | comma-delimited path fragments to exclude |
| `-o, --output <PATH>` | output path; extension picks format (`.md`/`.json`/`.sarif`) |
| `--highs-only` | high-severity detectors only |
| `--no-snippets` | omit code snippets |
| `--stdout` | print to stdout instead of writing a file |
| `--skip-cloc` | skip line-count analysis |
| `--auditor-mode` | emit manual-audit scaffolds instead of detector output |
| `-l, --lsp` | start the LSP server on stdio |
| `--mcp` | start the MCP server on stdio |
| `--config <PATH>` | path to `aderyn.toml` |

### Subcommands

```text
lancer-aderyn init [PATH]           # write an aderyn.toml scaffold
lancer-aderyn registry [DETECTOR]   # list all detectors, or detail one
lancer-aderyn mcp <stdio|http-stream> [--port 6277]
lancer-aderyn docs [QUESTION]       # open the Cyfrin docs
lancer-aderyn completions <SHELL>   # bash/zsh/fish/powershell/elvish
```

### Source-level ignore directives

```solidity
uint256 x; // aderyn-ignore(detector-name)
// aderyn-ignore-next-line(detector-name)
block.timestamp;
transfer(x); // aderyn-fp(detector-name)   // known false positive (separate bucket)
```

### `aderyn.toml`

```toml
version = 1
src = "src/"
exclude = ["src/test/", "src/mocks/"]

[detectors]
include = []                  # opt-in subset (empty = run all)
exclude = ["todo"]            # opt-out subset
```

## Requirements

- [`solc`](https://docs.soliditylang.org/) on `PATH` (or set `SOLC_PATH`) for live
  analysis. Pre-built AST JSON can be analyzed with `--from-ast <file.json>`.
- [Rust / Cargo](https://rustup.rs) and (on Windows) Visual C++ Build Tools to build.

## Build

### Windows
```
build.bat
```
Output: `build\lancer-aderyn.exe` (double-click opens the GUI).

Packaged dist build:
```
build-windows.bat
```
Output: `dist\lancer-aderyn-windows-x86_64\bin\lancer-aderyn.exe`.

### Linux / macOS
```bash
./build-linux.sh     # -> build/lancer-aderyn
./build-mac.sh       # -> build/lancer-aderyn (+ optional .app bundle)
```

## Architecture

A single crate, internally mirroring upstream's module split:

- `ast/` — self-contained solc-JSON AST model (serde).
- `context/` — `WorkspaceContext` (typed node indexes, id→node map, parent links),
  browser helpers, call/inheritance graphs, flow primitives.
- `detect/` — the `IssueDetector` trait, `capture!` macro, registry, and the
  `high/` + `low/` detector collections (one file = one detector).
- `report/` — Markdown / JSON / SARIF emitters.
- `driver/` — project discovery, solc invocation, ignore-directive suppression,
  the analysis pipeline.
- `lsp/`, `mcp/`, `gui/` — the three alternate frontends over the same core.

## License

MIT.
