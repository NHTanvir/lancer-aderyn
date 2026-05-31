//! `--self-test` — runs the bundled parity matrix against a canned reference
//! target and prints pass/fail per detector group, without needing the upstream
//! binary or a live `solc` (it analyzes an embedded pre-built AST).

use std::path::PathBuf;

use crate::driver::{analyze, RunConfig};

/// The embedded reference AST (a small contract exercising several detectors).
const REFERENCE_AST: &str = include_str!("../tests/data/sample_standard_json.json");

/// Detectors that MUST fire on the reference target.
const EXPECTED_HIGH: &[&str] = &["tx-origin-used-for-auth", "weak-randomness"];

/// Run the self-test. Returns true if every check passed.
pub fn run() -> bool {
    println!("lancer-aderyn self-test");
    println!("=======================");
    println!("registered detectors: {}", crate::detect::detector_count());

    let tmp = std::env::temp_dir().join("lancer-aderyn-selftest.json");
    if let Err(e) = std::fs::write(&tmp, REFERENCE_AST) {
        println!("FAIL: could not stage reference AST: {e}");
        return false;
    }

    let mut cfg = RunConfig::new(tmp.clone());
    cfg.from_ast = Some(tmp);
    cfg.skip_cloc = true;

    let report = match analyze(&cfg) {
        Ok(r) => r,
        Err(e) => {
            println!("FAIL: analysis errored: {e}");
            return false;
        }
    };

    let fired: Vec<String> = report
        .all_issues()
        .map(|i| i.detector_name.clone())
        .collect();

    let mut all_pass = true;
    for expected in EXPECTED_HIGH {
        let ok = fired.iter().any(|n| n == expected);
        println!("  [{}] {expected}", if ok { "PASS" } else { "FAIL" });
        all_pass &= ok;
    }

    println!(
        "\nsummary: {} high, {} low issue(s) on the reference target",
        report.high_issue_count(),
        report.low_issue_count()
    );
    println!("result: {}", if all_pass { "PASS" } else { "FAIL" });
    all_pass
}
