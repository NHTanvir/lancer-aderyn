//! End-to-end pipeline tests using a pre-built AST fixture (no solc required).

use std::path::PathBuf;

use lancer_aderyn::driver::{analyze, RunConfig};
use lancer_aderyn::report::{render, RenderOptions, ReportFormat};

fn sample_cfg() -> RunConfig {
    let ast = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("sample_standard_json.json");
    let mut cfg = RunConfig::new(ast.clone());
    cfg.from_ast = Some(ast);
    cfg.skip_cloc = true;
    cfg
}

#[test]
fn analyzes_sample_ast_and_flags_high_issues() {
    let report = analyze(&sample_cfg()).expect("analysis should succeed on the fixture");
    let high_names: Vec<String> = report.highs.iter().map(|i| i.detector_name.clone()).collect();

    assert!(
        high_names.contains(&"tx-origin-used-for-auth".to_string()),
        "expected tx-origin finding, got {high_names:?}"
    );
    assert!(
        high_names.contains(&"weak-randomness".to_string()),
        "expected weak-randomness finding, got {high_names:?}"
    );
}

#[test]
fn renders_all_report_formats() {
    let report = analyze(&sample_cfg()).expect("analysis");
    let opts = RenderOptions::default();

    let md = render(&report, ReportFormat::Markdown, opts);
    assert!(md.contains("# Aderyn Analysis Report"));

    let json = render(&report, ReportFormat::Json, opts);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert!(parsed.get("high_issues").is_some());

    let sarif = render(&report, ReportFormat::Sarif, opts);
    let parsed: serde_json::Value = serde_json::from_str(&sarif).expect("valid sarif json");
    assert_eq!(parsed["version"], "2.1.0");
}

#[test]
fn highs_only_filters_low_issues() {
    let mut cfg = sample_cfg();
    cfg.highs_only = true;
    let report = analyze(&cfg).expect("analysis");
    assert!(report.lows.is_empty(), "highs-only should yield no low issues");
}
