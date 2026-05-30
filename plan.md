# RESEARCH DOSSIER: Aderyn

> Part of the Lancer ecosystem. A Rust clone of Cyfrin's Aderyn — Solidity static analyzer. Build with Claude Code.
> Upstream: github.com/Cyfrin/aderyn (Rust, 0.5.x → 0.6.x, GPL-3.0 from 2025, ~90 detectors). Ships built-in LSP + MCP server.

## Feature Parity Mandate

**`lancer-aderyn` MUST ship with 100% feature parity to the upstream `aderyn` tool — every flag, every subcommand, every output format, every probe / detector / module / payload / mode — plus a native Rust GUI on top.** This is not a "subset clone" — this is a full reimplementation in Rust with a GUI wrapper.

### Parity contract

- **Every CLI flag** documented in upstream `aderyn --help` (and the long-form man page / docs) must work in `lancer-aderyn --cli`.
- **Every output format** the upstream tool emits (text, JSON, XML, CSV, SARIF, custom formats) must round-trip through `lancer-aderyn` byte-equivalent where the format allows.
- **Every probe / detector / module / category / payload class / built-in script** must be ported. None silently dropped.
- **Every supported target type / input format / protocol / file format / language** must work.
- **Every configuration option** (config files, env vars, registry / settings entries) must be honoured.
- **Companion binaries / sub-tools** in the upstream suite (if any) must each get their own binary in this workspace.

### GUI on top

Every feature reachable via CLI is also reachable from the GUI (see `## GUI (Lancer-standard pattern)` below). The GUI is not a subset — it is an alternate frontend over the same `core` library that the CLI uses. The `core` crate is GUI-agnostic.

### Verification

- The `## Granular feature catalog` section below enumerates everything that must be supported.
- Reference: `tests/upstream_parity/` will pin upstream flag combinations + canned outputs as parity tests. CI fails if any flag is unsupported or output diverges.
- A `lancer-aderyn --self-test` flag runs the full parity matrix against a bundled reference target and prints pass/fail per upstream feature group.

### What is allowed to diverge

- **Performance** — Rust-native, async-first; we are allowed to be faster.
- **Memory footprint** — we are allowed to be smaller.
- **GUI** — entirely new (no upstream GUI to copy except where the upstream ships its own GUI like Zenmap / Wireshark UI / Ghidra GUI / jadx-gui — in which case our GUI must cover every panel the upstream GUI has, plus the Lancer-standard layout from `## GUI` below).
- **AI augmentation** — net-new (see `## AI augmentation ideas`).

### What is NOT allowed to diverge

- Any flag silently dropped, renamed, or behavior-changed compared to upstream.
- Any output format / record schema diverging in field names, types, or semantics.
- Any built-in script / detector / module not ported (mark explicitly with `[N/A — see note]` if genuinely impossible in Rust, e.g. NSE Lua scripts requiring features not in `mlua`).

## What it actually does

Aderyn (Cyfrin, GPL-3.0 from 2025) is a Rust-based Solidity static analyzer. It traverses the Solidity AST emitted by solc (via Cyfrin's `solidity-ast-rs` crate), runs ~90 named "detectors," and emits a markdown/SARIF/JSON report listing high- and low-severity issues. Differentiators vs Slither: pure Rust (no Python toolchain), fast even on large workspaces, ships as a single static binary, has a VS Code extension (`vscode-aderyn`) and built-in MCP server (so Claude/Cursor can call it as a tool), built-in LSP server (`tower-lsp` 0.20) for editor diagnostics. Active line is 0.5.x → 0.6.x.

## Internal architecture

**Workspace** (`Cargo.toml`):
```
[workspace]
members = ["aderyn", "aderyn_core", "aderyn_driver", "tools/*"]
```

- **`aderyn`** — CLI binary (clap 4.4.6). Parses args, picks LSP/MCP/CLI mode, calls `aderyn_driver`.
- **`aderyn_driver`** — Orchestration: foundry/hardhat project discovery, solc invocation (via `solidity-ast-rs`), report formatting (markdown/SARIF/JSON), config merging from `aderyn.toml`.
- **`aderyn_core`** — Analysis engine: AST types, WorkspaceContext builder, detector registry, browser helpers, callgraph, flow analysis, MCP module.
- **`tools/*`** — Subordinate utilities (Solidity AST dumper, detector scaffolding, etc.).

**Key modules in `aderyn_core/src/`:**
- `ast/` — Solidity AST node definitions matching solc's compiler output JSON (re-exported from `solidity-ast-rs`); `NodeID` is `i64`.
- `context/`:
  - `workspace.rs` — `WorkspaceContext` aggregating all parsed source units, with typed accessors: `contract_definitions()`, `function_definitions()`, `function_calls()`, `variable_declarations()`, `binary_operations()`, `member_accesses()`, `assignments()`, `source_units()`, etc.
  - `browser/` — `Peek` (extract source text for a node), `GetClosestAncestorOfTypeX`, `GetImmediateChildren`, `ApproximateStorageChangeFinder`.
  - `flow/` — Control- and data-flow primitives.
  - `graph/` — petgraph-backed callgraph and contract-inheritance graph.
  - `mcp.rs`, `router.rs` — MCP server endpoints exposing detectors as tools.
  - `capturable.rs` — Trait for nodes that can be reported (yields a (file, line, snippet) tuple).
- `detect/`:
  - `detector.rs` — `IssueDetector` trait + `IssueSeverity { Low, High }` enum + `define_detectors!` macro that auto-registers every detector struct.
  - `high/` (40 detectors) and `low/` (50 detectors) directories — one file = one detector.
  - `test_utils.rs` — `load_solidity_source_unit(path)` helper for unit tests.
  - `helpers/` — reusable predicates (`is_state_variable`, `function_visibility`, `has_modifier`, etc.).
- `report/` — Markdown/SARIF (via `serde-sarif`) emitters; Askama templates.
- `lsp/`, `mcp/` — `tower-lsp` 0.20 + custom MCP server.

**How detectors plug in:**
1. Each detector is a Rust struct implementing the trait:
```rust
pub trait IssueDetector: Send + Sync + 'static {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>>;
    fn severity(&self) -> IssueSeverity;
    fn title(&self) -> String;
    fn description(&self) -> String;
    fn name(&self) -> String;
    fn instances(&self) -> BTreeMap<(String, usize, String), NodeID>;
    fn hints(&self) -> BTreeMap<(String, usize, String), String> { BTreeMap::new() }
}
```
2. The `capture!(self, ctx, node)` macro inserts into `found_instances` keyed by `(file, line, snippet)` and binds the `NodeID`.
3. `define_detectors!` macro lists every struct → generates an enum + `get_all_issue_detectors()` + `request_issue_detector_by_name()`.
4. `aderyn_driver` iterates the detector list (parallelized with `rayon`), calling `detect()` on the shared `WorkspaceContext`.

**AST context building:**
1. `solidity-ast-rs` parses solc's JSON output (`storageLayout`, `metadata`, `abi`, `ast` blocks).
2. `WorkspaceContext::new()` walks every source unit, registering nodes into typed indexes (vector of `&ContractDefinition`, `&FunctionDefinition`, etc.) and a `NodeID → Node` map.
3. Parent links and inheritance graph are built once (callgraph: petgraph `DiGraph<NodeID, ()>`).
4. Detectors operate on the immutable context concurrently.

## Granular feature list

**CLI:** `aderyn [PATH]` plus flags — `--root`, `--src`, `--scope`, `--exclude`, `--report-out` (`report.md`), `--report-format` (`markdown`/`json`/`sarif`), `--highs-only`, `--no-snippets`, `--lsp`, `--mcp`, `--auditor-mode`, `--stdout`, `--config` (`aderyn.toml`), `--include-paths`, `--skip-update-check`, `--auto-include-libs`.

**CLI (verified 0.6.8 surface — short / long forms + subcommands):**
- Positional `[ROOT]` (default `.`) — Foundry/Hardhat project root, or single `.sol` file.
- `-s, --src <SRC>` — source contracts dir (auto-captured from `foundry.toml`; `--src=contracts/` for Hardhat).
- `-i, --path-includes <CSV>` — comma-delimited path-fragments to explicitly include (case-sensitive, no spaces).
- `-x, --path-excludes <CSV>` — comma-delimited path-fragments to exclude.
- `-o, --output <PATH>` — output file path; extension chooses format (`.md` / `.json` / `.sarif`). Default `report.md`.
- `-l, --lsp` — start LSP server on stdout (was BETA until 0.6.7; promoted to GA).
- `--no-snippets` — omit code snippets (reduces report size on large repos).
- `--highs-only` — high-severity detectors only.
- `--stdout` — print report to stdout instead of writing the output file.
- `--skip-cloc` — skip cloc/line-count analysis.
- `--skip-update-check` — disable version-check call to GitHub releases.
- `--auditor-mode` — output manual-audit helpers only (suppresses normal detector output, enables the auditor scaffolds in `aderyn_core/src/audit/`).
- `-h, --help` / `-V, --version`.
- Subcommand `init [PATH]` — write `aderyn.toml` scaffold to root (or a subdir).
- Subcommand `registry [DETECTOR]` — browse all detectors (`all`) or print full detail for one detector name.
- Subcommand `mcp <TRANSPORT>` — start MCP server; `mcp stdio` for editor/Claude-Desktop usage, `mcp http-stream --port 6277` for IDE / Cursor / remote-agent usage. (MCP promoted out of beta in 0.6.7.)
- Subcommand `docs [QUESTION]` — open Cyfrin docs in browser, with optional natural-language question forwarded to the docs AI search.
- Subcommand `completions <SHELL>` — emit bash/zsh/fish/powershell/elvish completion scripts (added 0.6.1, via `clap_complete`).
- Subcommand `help [SUBCOMMAND]` — clap auto-help.

**Source-level ignore directives (must be honored by the scanner):**
- `// aderyn-ignore-next-line(detector-name)` — suppress one finding on the next line.
- `// aderyn-ignore(detector-name)` — suppress one finding on the current line.
- `// aderyn-fp(detector-name)` — mark a known false positive (separate bucket from "ignore" — surfaces in a dedicated "FP review" section of the report).

**Config (`aderyn.toml`):**
```toml
src = "contracts/"
remappings = ["@oz/=lib/oz/"]
[detectors]
exclude = ["todo", "empty-block"]
[detectors.include]
custom = "path/to/custom_detector.dylib"
```

**Config (`aderyn.toml`) — verified 0.6.x schema (additive over the snippet above):**
```toml
version = 1                          # schema version (required)
root = "."
src = "src/"
include = ["src/file.sol", "src/Lib.sol"]    # explicit allow-list
exclude = ["src/test/", "src/mocks/"]         # explicit deny-list

[env]                                # env-vars exported into the solc + foundry sub-processes
FOUNDRY_PROFILE = "default"

[detectors]
include = ["weak-randomness", "tx-origin-used-for-auth"]   # opt-in subset (overrides default-all)
exclude = ["todo", "empty-block"]                          # opt-out subset
```

Notes:
- Default behaviour: all detectors run, all files in `src` are included.
- `[env]` is honoured by the foundry-compilers crate when the driver shells out to `solc` / `forge`.
- Schema is versioned (`version = 1`) so future bumps can stay backwards-compatible.

**Install / distribution channels (parity with upstream):**
- `cyfrinup` — first-party Cyfrin tool-manager (cross-platform install/upgrade across the Cyfrin suite). `lancer-aderyn` must publish a `lancerup` equivalent (or be discoverable by `cyfrinup` via a fork) so the install story is one-liner.
- `curl ... aderyn-installer.sh | bash` — single-shot bootstrap script.
- `brew install cyfrin/tap/aderyn` — Homebrew tap.
- `npm install -g @cyfrin/aderyn` — npm wrapper.
- Direct GitHub release binaries per platform.
- Note: upstream officially requires WSL on Windows. `lancer-aderyn` MUST also run native Windows (Lancer rule — single `.exe` from `build.bat`).

**High-severity detectors (40)** (file ↔ detector mapping from `aderyn_core/src/detect/high/`):
1. `abi_encode_packed_hash_collision` — encodePacked with multiple dynamic types
2. `arbitrary_transfer_from` — transferFrom with unchecked `from` arg
3. `const_func_changes_state` — view/pure function modifies state
4. `contract_locks_ether` — receive/payable but no withdrawal
5. `dangerous_unary_operator` — `=+` / `=-` (intended `+=`/`-=`)
6. `delegate_call_unchecked_address` — delegatecall to user-controlled address
7. `delete_nested_mapping` — `delete` on a struct containing a mapping (no-op)
8. `dynamic_array_length_assignment` — direct write to `.length`
9. `enumerable_loop_removal` — modifying iterated collection
10. `eth_send_unchecked_address` — call.value to arbitrary address
11. `experimental_encoder` — `pragma experimental ABIEncoderV2`
12. `function_selector_collision` — two function names with same 4-byte selector
13. `incorrect_caret_operator` — `^` (XOR) used as exponentiation
14. `incorrect_erc20_interface` — wrong function signatures vs ERC-20
15. `incorrect_erc721_interface` — wrong signatures vs ERC-721
16. `incorrect_shift_order` — `2 << x` instead of `x << 2`
17. `misused_boolean` — `||` where `&&` intended (and vice versa)
18. `msg_value_in_loops` — using msg.value inside a loop
19. `multiple_constructors` — two constructor-like functions
20. `nested_struct_in_mapping` — pre-0.5 cannot push to nested
21. `out_of_order_retryable` — L2 retryable misuse
22. `pre_declared_variable_usage` — variable used before declaration
23. `reentrancy_state_change` — state write after external call
24. `reused_contract_name` — two contracts with same name in scope
25. `rtlo` — right-to-left-override char in source
26. `selfdestruct` — uses deprecated SELFDESTRUCT
27. `signed_integer_storage_array` — int storage arrays gotcha
28. `state_variable_shadowing` — child shadows parent state
29. `storage_array_memory_edit` — modifying memory copy expecting storage
30. `strict_equality_contract_balance` — `address(this).balance == X`
31. `tautological_compare` — `x == x`
32. `tautology_or_contradiction` — always-true or always-false condition
33. `tx_origin_used_for_auth` — `tx.origin == X` for auth
34. `unchecked_low_level_call` — `addr.call()` without checking return
35. `unchecked_send` — `addr.send()` without checking return
36. `unprotected_initializer` — initializer with no `initializer` modifier or access guard
37. `unsafe_casting` — narrowing cast without bound check
38. `weak_randomness` — `block.timestamp`/`blockhash`/`prevrandao` used as randomness
39. `yul_return` — Yul `return` bypassing modifiers
40. `_template` — scaffolding (not active)

**Low-severity detectors (50):**
1. `assert_state_change` — `assert(...)` with side effects
2. `block_timestamp_deadline` — using `block.timestamp` as deadline
3. `boolean_equality` — `x == true` / `x == false`
4. `builtin_symbol_shadowing` — shadowing `now`, `revert`, etc.
5. `centralization_risk` — `Ownable` patterns
6. `constant_function_contains_assembly` — pure/view with inline asm
7. `costly_loop` — SSTORE in loop
8. `dead_code` — unreachable
9. `delegatecall_in_loop` — delegatecall inside loop
10. `deprecated_oz_function` — OZ deprecated APIs
11. `division_before_multiplication` — precision-loss pattern
12. `ecrecover` — direct `ecrecover` (signature malleability)
13. `empty_block` — empty function bodies
14. `empty_require_revert` — `require(x, "")` / `revert("")`
15. `function_initializing_state` — state init in a regular fn instead of constructor
16. `function_pointer_in_constructor` — fn ptr param in constructor (pre-0.5)
17. `inconsistent_type_names` — mixing `uint`/`uint256`
18. `incorrect_modifier` — modifier missing `_`
19. `internal_function_used_once` — internal fn called only once
20. `large_numeric_literal` — `100000000000000` should be `100e18`
21. `literal_instead_of_constant` — magic numbers
22. `local_variable_shadowing` — local shadows state
23. `missing_inheritance` — implements but doesn't inherit interface
24. `modifier_used_only_once`
25. `multiple_placeholders` — multiple `_` in modifier
26. `non_reentrant_not_first` — `nonReentrant` not first modifier
27. `push_0_opcode` — uses PUSH0 on chains that don't support it
28. `redundant_statement`
29. `require_revert_in_loop` — revert inside loop blocking others
30. `return_bomb` — assigning low-level call return without size cap
31. `solmate_safe_transfer_lib` — solmate misuse
32. `state_change_without_event` — setter without event emission
33. `state_no_address_check` — setter doesn't validate non-zero address
34. `state_variable_could_be_constant`
35. `state_variable_could_be_immutable`
36. `state_variable_init_order` — via_ir reorder (added 2025)
37. `state_variable_read_external` — external SLOAD pattern
38. `storage_array_length_not_cached`
39. `todo` — `// TODO` comments
40. `unchecked_return`
41. `uninitialized_local_variable`
42. `unsafe_erc20_operation` — IERC20.transfer without SafeERC20
43. `unsafe_oz_erc721_mint` — `_mint` instead of `_safeMint`
44. `unspecific_solidity_pragma` — `^0.8.0` floating
45. `unused_error`
46. `unused_import`
47. `unused_public_function`
48. `unused_state_variable`
49. `void_constructor` — constructor calls inherited constructor with no args
50. `_template`

**Custom detectors:** Users implement `IssueDetector`, drop into a folder, register in `detectors.rs` (compile-time) or load dylib at runtime.

## Engineering hard parts

**AST traversal.** Detectors don't recursively walk — they query the pre-indexed `WorkspaceContext` accessors. This trades memory (everything held) for O(1) lookup per category. The hard parts:
- Some queries need ancestor lookup (`GetClosestAncestorOfTypeX`) which requires walking parent pointers built in the indexing step.
- Cross-contract analysis (e.g., `arbitrary_transfer_from` must follow an external call to a known IERC20) needs the inheritance graph and resolved function selectors.

**False-positive suppression.** Aderyn does NOT auto-suppress; it relies on:
- Per-detector exclusion via `aderyn.toml`.
- Per-instance NatSpec hints (`@aderyn-ignore-next-line foo`) checked by capture macro.
- The `auditor mode` flag enabling broader detectors (more FPs, more coverage).

**Severity classification.** Just two buckets (`High`, `Low`). No critical/medium gradient; tradeoff to keep semantics unambiguous. High = "very likely real exploitable issue or correctness bug"; Low = "code quality / minor risk."

**Reentrancy state-change detector** (canonical hard detector):
1. For every external call (CALL/STATICCALL targeting non-precompile), find the enclosing function.
2. Walk subsequent statements in the function body (after the call).
3. Use `ApproximateStorageChangeFinder` to check whether any later SSTORE-effective statement exists (assignments to state vars, +=/-= on state).
4. Capture if found; suppress if `nonReentrant` modifier on the function.

**Performance via rayon.** Detectors run with `rayon::join` / `par_iter` over the detector list, contention only on the shared context (which is read-only after build). For 100k-line projects, full scan completes in seconds.

## Library mapping (Rust)

| Need | Crate | Version |
|---|---|---|
| Solidity AST parsing | `solidity-ast` (from `solidity-ast-rs`) | git tag v0.0.1-alpha.beta.7 |
| Graph algorithms | `petgraph` | 0.x |
| Templating | `askama` | 0.14 |
| Async runtime | `tokio` | 1.40 |
| LSP server | `tower-lsp` | 0.20 |
| CLI | `clap` | 4.4.6 |
| Parallelism | `rayon` | 1.11 |
| Errors | `eyre` | 0.6.12 |
| SARIF output | `serde-sarif` | 0.4.2 |
| JSON | `serde_json` | 1.0.96 |
| Logging | `tracing` | 0.1.x |

(For a clone: substitute `solar-parse` + `solar-sema` from the Foundry stack if you want to avoid the Cyfrin AST crate.)

## Validation test corpus

- **SmartBugs Curated** — 9 DASP categories (Reentrancy, Access Control, Arithmetic, Unchecked Low-Level Calls, DoS, Bad Randomness, Front Running, Time Manipulation, Short Addresses). Each contract annotated with vulnerability line numbers — perfect detector ground truth.
- **Aderyn's `tests/contract-playground/`** — 100+ contrived Solidity contracts, each exercising one detector. Used as regression suite.
- **`reports/playground-report.md`** in the repo — canonical golden report.
- **OpenZeppelin upgrades samples** — for `unprotected_initializer`, `state_variable_init_order` (via_ir).
- **Real audit findings imported** — Cyfrin maintains a list of public audits where a given finding maps to the detector that would catch it.

## 2024–2026 state

- **0.5.x:** Initial public release with ~50 detectors; report-only CLI.
- **0.5.x → 0.6.x (2025):** doubled detector count to ~90; added `unprotected_initializer`, `weak_randomness`, `function_selector_collision`, `reentrancy_state_change` (rewritten); LSP server (`tower-lsp` 0.20); VS Code extension; MCP server module so Claude/Cursor can run detectors as a tool; SARIF output for GitHub code-scanning; new detectors for via_ir compile mode (`state_variable_init_order`); `state_change_without_event`; `state_no_address_check`. Switched to MIT/GPL-3.0 dual licensing on core.
- **2026 dev (latest):** new detectors landing around L2 retryables (`out_of_order_retryable`), `signed_integer_storage_array`, expanded ERC-20/ERC-721 conformance checks. AI-augmented custom-detector workflow (LLM scaffolds detector from natural-language rule).

## Real-world audit workflows

- **Pre-audit triage.** First tool run before opening any file. The "Files" section of the report becomes the auditor's scope index. Detector counts per file proxy for risk hotspots.
- **CI integration.** Cyfrin recommends `aderyn .` in pre-commit and GitHub Actions; SARIF uploaded to security tab. Many Code4rena scoping repos include this baseline.
- **Cross-tool.** Aderyn cheap → run on every save in VS Code. Slither for deeper dataflow (taint, modifiers). Mythril for symbolic. Then Foundry/Medusa fuzz, Halmos symbolic. Solodit checklist for sanity.
- **Custom detector workflow.** For a contest where a known anti-pattern exists, an auditor scaffolds a custom Rust detector once, then rolls it into all subsequent audits.

## AI augmentation ideas

- **Custom detector generation.** "Write me a detector that flags any function calling `_burn` while `paused == true`." LLM emits Rust code using `WorkspaceContext` APIs.
- **False-positive suppression.** Each finding goes to Claude with surrounding context for a confidence rating. Below threshold = auto-suppressed in report.
- **Severity uplifting.** LLM reads context (whose tokens, whose balance) and proposes whether a "low" should be promoted (e.g., centralization risk in a token holding $50M).
- **Auto-fix.** Given a finding + AST snippet, propose a patch.
- **Finding clustering.** Across a 30-finding report, group semantically (e.g., "all 12 of these are the same missing-event pattern").

## Implementation phases (10-week clone)

**Week 1 — Workspace & AST.** Cargo workspace (`aderyn`, `aderyn_core`, `aderyn_driver`). Decide parser strategy: either fork `solidity-ast-rs` or wrap `solar-parse`+`solar-sema`. Get JSON AST → typed Rust nodes round-tripping for OpenZeppelin's ERC20.sol.

**Week 2 — WorkspaceContext.** Build typed indexes: `contract_definitions`, `function_definitions`, `function_calls`, `variable_declarations`, `binary_operations`, `member_accesses`, `assignments`, `source_units`. Parent pointers. Inheritance graph via petgraph.

**Week 3 — Detector trait + 10 high detectors.** Implement `IssueDetector`, `capture!` macro, `define_detectors!` macro. Build first 10: `weak_randomness`, `tx_origin_used_for_auth`, `unchecked_low_level_call`, `unchecked_send`, `arbitrary_transfer_from`, `delegate_call_unchecked_address`, `selfdestruct`, `experimental_encoder`, `multiple_constructors`, `pre_declared_variable_usage`. Set up regression tests against SmartBugs-curated subset.

**Week 4 — 15 more high detectors + reentrancy.** Implement remaining purely-syntactic ones, then `reentrancy_state_change` requiring approximate storage-change analysis.

**Week 5 — Low detectors (25 of 50).** Code-quality (unused-state, unused-import, empty-block, todo, boolean-equality, division-before-multiplication, unspecific-pragma, centralization-risk, state-change-without-event, etc.).

**Week 6 — Low detectors (other 25) + browser helpers.** `GetClosestAncestorOfTypeX`, `Peek` (source extraction), helper predicates. Round-trip SARIF output.

**Week 7 — CLI + Foundry/Hardhat project discovery.** `aderyn_driver` walks `foundry.toml`/`hardhat.config`, invokes solc with right remappings, collects all source units. Markdown report templating (Askama). SARIF emitter.

**Week 8 — LSP server.** `tower-lsp` 0.20; expose detectors as `textDocument/publishDiagnostics`. Test in VS Code dev host.

**Week 9 — MCP server.** Expose each detector as an MCP tool (`run_detector`, `list_detectors`, `scan_path`). Wire into Claude.

**Week 10 — Custom detector loading + AI integration.** dylib loader for runtime detector plugins, plus an MCP tool that takes a natural-language rule, asks an LLM to scaffold a detector, compiles it. Performance pass with `rayon` parallelization.

---

## GUI (Lancer-standard pattern)

`lancer-aderyn` ships as a **native GUI application** by default. The CLI surface listed in `## Granular feature catalog` is still available behind a `--cli` flag (or via the `lancer-aderyn-cli` companion binary), but the **plain `lancer-aderyn` invocation opens a native desktop window**, matching every other tool in the Lancer ecosystem. No terminal-only mode.

### Framework

Pure-Rust GUI stack — no JavaScript, no Node, single static binary:

| Crate | Role |
|-------|------|
| `eframe` | Cross-platform window + event loop wrapper around `egui` |
| `egui` | Immediate-mode UI widgets (panels, tables, forms, menus, drag-and-drop) |
| `egui_extras` | Sortable tables, image loaders, syntax highlighting, date pickers |
| `egui_plot` | Charts, graphs, timelines (histograms, activity meters, progress) |
| `rfd` | Native file/folder pickers (open file, save dialog) |
| `directories` | Per-OS config dir (`~/.config/lancer-aderyn/`, `%APPDATA%\lancer-aderyn\`, etc.) |
| `image` | Screenshot / preview rendering inside egui |
| `arboard` | Clipboard read/write for "copy result" actions |
| `muda` | Native menu bar (Linux / macOS / Windows) |
| `egui-notify` | Toast notifications |

Optional dark/light theme persisted to `lancer-aderyn-config.json` via `serde_json`.

### Window layout

Three-pane layout matching the Lancer mothership GUI:

```
┌─────────────────────────────────────────────────────────────────────┐
│  File   Tools   Settings   Help                              [≡]    │
├──────────┬────────────────────────────────────────┬─────────────────┤
│          │                                        │                 │
│  Inputs  │           Main view                    │  Detail pane    │
│  / jobs  │  (table of results / source tree /     │  (full record / │
│  / scope │   findings list / packet list / etc.)  │   bytes / src / │
│          │                                        │   remediation)  │
│          │                                        │                 │
├──────────┴────────────────────────────────────────┴─────────────────┤
│  Status: idle           Items: 0          [Open]  [Run]  [Export]   │
└─────────────────────────────────────────────────────────────────────┘
```

- **Left sidebar (200-300px)** — inputs / queued jobs / scope / target list. Drag-and-drop accepted.
- **Center pane (flexible)** — primary result view appropriate to the tool (table for findings, tree for hierarchical data, code editor for source).
- **Right pane (300-400px, collapsible)** — full detail of the currently-selected item.
- **Top menu bar** — File (Open / Save / Export / Recent / Quit), Tools (per-tool actions), Settings (theme, fonts, integrations), Help (About / Docs).
- **Bottom status bar** — current activity, item counts, primary action buttons.

### Cross-tool integration with Lancer + the other 29 clones

Every Lancer-clone GUI exposes a small local HTTP server on a port auto-allocated at launch (`127.0.0.1:<port>`, written to `%TEMP%/lancer-aderyn.port`):

- The main Lancer GUI discovers running tools by scanning `%TEMP%/lancer-*.port` and lists them in its sidebar.
- Right-clicking any item in any tool can "Send to <other Lancer tool>" — the receiving tool fetches the payload via its own HTTP endpoint.
- A future "Lancer Hub" tray shell can launch / focus any of the 30 GUIs from one menu.
- HTTP endpoints exposed:
  - `GET  /v1/state` — current jobs / progress
  - `POST /v1/import` — accept a target / file / record from another Lancer tool
  - `GET  /v1/export?format=json|csv|sarif|markdown` — pull the current result set in a portable format

### Async work + responsiveness

- Long-running scans run on a Tokio task pool; the GUI thread only renders.
- Progress propagates via `crossbeam::channel` from worker → UI.
- A `Stop` button cancels in-flight work via a `CancellationToken`.
- All file I/O / network I/O / subprocess work goes through async channels — the egui thread never blocks.

### Build output

The existing `build.bat` / `build-windows.bat` / `build-linux.sh` already produce a single binary. With `eframe` linked, the same binary is the GUI app.

- Windows: `build\lancer-aderyn.exe` (double-click opens the GUI window).
- Linux: `build/lancer-aderyn` (also installable as a `.desktop` entry via `build-linux.sh --install`).
- macOS: `build/lancer-aderyn` (bundled into `lancer-aderyn.app` via `cargo-bundle` in the `build-mac.sh` companion).

For CI / headless / scripting, pass `--cli` to fall through to the CLI surface from `## Granular feature catalog`.

## Build Setup (Lancer-standard pattern)

This clone follows the same build pattern as the main Lancer project — single `.exe` output via `build.bat` on Windows, `.so/elf` via `build-linux.sh` on Linux.

### Project layout

```
lancer-aderyn/
├── Cargo.toml              # single-crate Rust project
├── Cargo.lock              # committed (binary, not library)
├── src/
│   └── main.rs             # entry point (scaffold)
├── build.bat               # Windows build → build/lancer-aderyn.exe
├── build-windows.bat       # Windows packaged dist build → dist/lancer-aderyn-windows-x86_64/
├── build-linux.sh          # Linux build → build/lancer-aderyn
├── .gitignore              # plan.md (local-only) + target/ + build/ + dist/
├── README.md               # public-facing (pushed to GitHub)
└── plan.md                 # this file (gitignored — local blueprint, never pushed)
```

### Build commands

**Windows (double-click `build.bat`):**
```
build.bat
```
Produces `build\lancer-aderyn.exe`. Pauses at end so the window stays open.

**Windows (packaged dist):**
```
build-windows.bat
```
Produces `dist\lancer-aderyn-windows-x86_64\bin\lancer-aderyn.exe` plus LICENSE/README — ready to zip.

**Linux / macOS:**
```bash
./build-linux.sh
```
Produces `build/lancer-aderyn`.

### Requirements
- Rust toolchain (https://rustup.rs)
- Windows: Visual C++ Build Tools 2019/2022 (needed by some crates)

### CI

The build pattern is intentionally identical across Lancer and all 19 clones, so a single Jenkins/GitHub Actions matrix can build everything by calling `build-windows.bat` or `build-linux.sh` per repo.
