//! Tests for `aderyn.toml` parsing + detector enable/disable logic.

use lancer_aderyn::config::AderynConfig;

#[test]
fn parses_verified_schema() {
    let toml = r#"
version = 1
root = "."
src = "src/"
include = ["src/Token.sol"]
exclude = ["src/test/"]
remappings = ["@oz/=lib/oz/"]

[env]
FOUNDRY_PROFILE = "default"

[detectors]
include = ["weak-randomness", "tx-origin-used-for-auth"]
exclude = ["todo"]
"#;
    let cfg: AderynConfig = toml::from_str(toml).expect("valid aderyn.toml");
    assert_eq!(cfg.version, Some(1));
    assert_eq!(cfg.src.as_deref(), Some("src/"));
    assert_eq!(cfg.include, vec!["src/Token.sol"]);
    assert_eq!(cfg.exclude, vec!["src/test/"]);
    assert_eq!(cfg.remappings, vec!["@oz/=lib/oz/"]);
    assert_eq!(cfg.env.get("FOUNDRY_PROFILE").map(String::as_str), Some("default"));
}

#[test]
fn include_subset_overrides_default_all() {
    let toml = r#"
[detectors]
include = ["weak-randomness"]
"#;
    let cfg: AderynConfig = toml::from_str(toml).unwrap();
    assert!(cfg.detector_enabled("weak-randomness"));
    assert!(!cfg.detector_enabled("todo"));
}

#[test]
fn exclude_subset_opts_out() {
    let toml = r#"
[detectors]
exclude = ["todo", "empty-block"]
"#;
    let cfg: AderynConfig = toml::from_str(toml).unwrap();
    assert!(!cfg.detector_enabled("todo"));
    assert!(!cfg.detector_enabled("empty-block"));
    assert!(cfg.detector_enabled("weak-randomness"));
}

#[test]
fn default_runs_everything() {
    let cfg = AderynConfig::default();
    assert!(cfg.detector_enabled("any-detector-name"));
}
