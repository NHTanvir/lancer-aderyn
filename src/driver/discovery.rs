//! Foundry / Hardhat project discovery + Solidity source collection.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use walkdir::WalkDir;

/// What kind of project root we're looking at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectKind {
    Foundry,
    Hardhat,
    SingleFile,
    Bare,
}

#[derive(Debug, Default, Deserialize)]
struct FoundryTomlProfile {
    src: Option<String>,
    #[serde(default)]
    remappings: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct FoundryToml {
    #[serde(default)]
    profile: std::collections::BTreeMap<String, FoundryTomlProfile>,
}

/// Resolved project layout.
#[derive(Debug, Default)]
pub struct ProjectLayout {
    pub kind_label: String,
    pub src_dir: Option<String>,
    pub remappings: Vec<String>,
}

pub fn detect_kind(root: &Path) -> ProjectKind {
    if root.is_file() && root.extension().map(|e| e == "sol").unwrap_or(false) {
        return ProjectKind::SingleFile;
    }
    if root.join("foundry.toml").exists() {
        return ProjectKind::Foundry;
    }
    if root.join("hardhat.config.js").exists() || root.join("hardhat.config.ts").exists() {
        return ProjectKind::Hardhat;
    }
    ProjectKind::Bare
}

/// Read `src` + `remappings` from `foundry.toml` (default profile).
pub fn read_foundry_layout(root: &Path) -> ProjectLayout {
    let mut layout = ProjectLayout {
        kind_label: "foundry".to_string(),
        ..Default::default()
    };
    let path = root.join("foundry.toml");
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(parsed) = toml::from_str::<FoundryToml>(&text) {
            if let Some(default) = parsed.profile.get("default") {
                layout.src_dir = default.src.clone();
                layout.remappings = default.remappings.clone();
            }
        }
    }
    // Foundry's remappings.txt takes precedence if present.
    if let Ok(text) = std::fs::read_to_string(root.join("remappings.txt")) {
        let extra: Vec<String> = text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && l.contains('='))
            .map(|l| l.to_string())
            .collect();
        if !extra.is_empty() {
            layout.remappings = extra;
        }
    }
    layout
}

/// Collect `.sol` files under `root`/`src`, honoring include/exclude fragments.
pub fn collect_solidity_files(
    root: &Path,
    src_dir: Option<&str>,
    includes: &[String],
    excludes: &[String],
) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.to_path_buf()];
    }

    let base = match src_dir {
        Some(s) => root.join(s),
        None => root.to_path_buf(),
    };
    let scan_root = if base.exists() { base } else { root.to_path_buf() };

    let mut files = Vec::new();
    for entry in WalkDir::new(&scan_root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().map(|e| e != "sol").unwrap_or(true) {
            continue;
        }
        let path_str = path.to_string_lossy().replace('\\', "/");

        // Default-skip common dependency/output dirs.
        if path_str.contains("/lib/")
            || path_str.contains("/node_modules/")
            || path_str.contains("/out/")
            || path_str.contains("/cache/")
            || path_str.contains("/artifacts/")
        {
            // Unless explicitly included.
            if !includes.iter().any(|inc| path_str.contains(inc)) {
                continue;
            }
        }

        if !includes.is_empty() && !includes.iter().any(|inc| path_str.contains(inc)) {
            continue;
        }
        if excludes.iter().any(|exc| path_str.contains(exc)) {
            continue;
        }
        files.push(path.to_path_buf());
    }
    files.sort();
    files
}

/// Compute the report-facing relative path for a source file.
pub fn relative_path(root: &Path, file: &Path) -> String {
    let rel = file.strip_prefix(root).unwrap_or(file);
    rel.to_string_lossy().replace('\\', "/")
}
