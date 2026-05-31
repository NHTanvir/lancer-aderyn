# Upstream parity tests

This directory pins the behaviour `lancer-aderyn` must match against upstream
`aderyn` (the parity contract in `plan.md`). These tests are **opt-in** — they
require both `solc` and a reference `aderyn` binary on `PATH`, so they are
`#[ignore]`d in normal `cargo test` runs and executed explicitly:

```bash
cargo test --test upstream_parity -- --ignored
```

## What is pinned

1. **Flag surface** — every flag in upstream `aderyn --help` must parse and behave
   identically (`tests/cli_surface.rs` checks the flag set is present).
2. **Output formats** — Markdown / JSON / SARIF reports must carry the same record
   schema (field names, severities) for a fixed input project.
3. **Detector coverage** — the registry must expose every upstream detector name
   (see `tests/registry.rs::expected_active_detector_count`).

## Self-test

`lancer-aderyn --self-test` runs the bundled parity matrix against
`tests/contract-playground/` and prints pass/fail per detector group, without
needing the upstream binary. Use it as a quick local sanity check.
