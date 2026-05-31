//! `aderyn.toml` configuration (verified 0.6.x schema).

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

/// The parsed `aderyn.toml`.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct AderynConfig {
    /// Schema version (required by upstream; we tolerate its absence).
    #[serde(default)]
    pub version: Option<u32>,
    #[serde(default)]
    pub root: Option<String>,
    #[serde(default)]
    pub src: Option<String>,
    /// Explicit allow-list of path fragments.
    #[serde(default)]
    pub include: Vec<String>,
    /// Explicit deny-list of path fragments.
    #[serde(default)]
    pub exclude: Vec<String>,
    /// solc import remappings (`@oz/=lib/oz/`).
    #[serde(default)]
    pub remappings: Vec<String>,
    /// Env-vars exported into solc / forge sub-processes.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Detector include/exclude subsets.
    #[serde(default)]
    pub detectors: DetectorConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct DetectorConfig {
    /// Opt-in subset (overrides default-all).
    #[serde(default)]
    pub include: Vec<String>,
    /// Opt-out subset.
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl AderynConfig {
    /// Load `aderyn.toml` from `root` (returns default if it does not exist).
    pub fn load_from_root(root: &Path) -> eyre::Result<AderynConfig> {
        let path = root.join("aderyn.toml");
        Self::load_from_file(&path)
    }

    pub fn load_from_file(path: &Path) -> eyre::Result<AderynConfig> {
        if !path.exists() {
            return Ok(AderynConfig::default());
        }
        let text = std::fs::read_to_string(path)?;
        let cfg: AderynConfig = toml::from_str(&text)?;
        Ok(cfg)
    }

    /// Whether `detector_name` is enabled given include/exclude rules.
    pub fn detector_enabled(&self, detector_name: &str) -> bool {
        if !self.detectors.include.is_empty() {
            return self.detectors.include.iter().any(|n| n == detector_name);
        }
        !self.detectors.exclude.iter().any(|n| n == detector_name)
    }
}

/// The default `aderyn.toml` scaffold written by `aderyn init`.
pub fn scaffold_toml() -> String {
    r#"# aderyn.toml — lancer-aderyn configuration
version = 1
root = "."
# src = "src/"

# Explicit allow-list / deny-list of path fragments (optional).
# include = ["src/"]
# exclude = ["src/test/", "src/mocks/"]

# solc import remappings (optional).
# remappings = ["@openzeppelin/=lib/openzeppelin-contracts/"]

# Env-vars exported into the solc / forge sub-processes (optional).
# [env]
# FOUNDRY_PROFILE = "default"

[detectors]
# Opt-in subset (overrides default-all). Leave empty to run all detectors.
include = []
# Opt-out subset.
exclude = []
"#
    .to_string()
}
