//! `push-zero-opcode` — flags Solidity pragmas that permit versions >= 0.8.20,
//! which cause the compiler to emit the PUSH0 opcode not supported on all EVM chains.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct Push0OpcodeDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Parse a bare version token like "0.8.20" into (major, minor, patch).
/// Returns `None` if the token does not look like a three-part semver.
fn parse_version(token: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;
    Some((major, minor, patch))
}

/// Returns `true` if `version` is >= (0, 8, 20).
fn version_gte_0_8_20(version: (u32, u32, u32)) -> bool {
    version >= (0, 8, 20)
}

/// Scan the joined pragma literals for any version token that is >= 0.8.20.
/// Tokens that start with a digit are treated as bare version strings.
fn pragma_allows_push0(joined: &str) -> bool {
    for token in joined.split_whitespace() {
        // Strip leading constraint characters (^, ~, >=, <=, >, <, =).
        let stripped = token.trim_start_matches(|c: char| {
            matches!(c, '^' | '~' | '>' | '<' | '=')
        });
        if let Some(ver) = parse_version(stripped) {
            if version_gte_0_8_20(ver) {
                return true;
            }
        }
    }
    false
}

impl IssueDetector for Push0OpcodeDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for pragma in context.pragma_directives() {
            // Only inspect `pragma solidity ...` directives.
            if pragma.literals.first().map(|s| s.as_str()) != Some("solidity") {
                continue;
            }
            let joined = pragma.literals.join(" ");
            if pragma_allows_push0(&joined) {
                capture!(self, context, pragma);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "PUSH0 May Be Unsupported".to_string()
    }

    fn description(&self) -> String {
        "Solidity >= 0.8.20 emits the PUSH0 opcode, which is not available on all EVM-compatible \
         chains (e.g. zkSync, older L2s). Pin the pragma to a version below 0.8.20, or confirm \
         that every deployment target supports the Shanghai EVM revision."
            .to_string()
    }

    fn name(&self) -> String {
        "push-zero-opcode".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
