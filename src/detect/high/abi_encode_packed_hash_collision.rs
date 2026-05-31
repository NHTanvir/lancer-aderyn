//! `abi-encode-packed-hash-collision` — flags abi.encodePacked calls with 2+ dynamic-type arguments that can collide when hashed.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct AbiEncodePackedHashCollisionDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if the type string denotes a dynamic ABI type that can cause
/// hash collisions when packed adjacent to another dynamic type.
///
/// Dynamic types: arrays (contain "[]"), bare `bytes` (not a fixed `bytesN`),
/// and `string`. Location qualifiers like "memory"/"storage"/"calldata" are
/// stripped before the check.
fn is_dynamic_type(type_string: Option<&str>) -> bool {
    let ts = match type_string {
        Some(s) => s,
        None => return false,
    };

    // Strip trailing location qualifier, e.g. "bytes memory" -> "bytes".
    let base = ts
        .strip_suffix(" memory")
        .or_else(|| ts.strip_suffix(" storage"))
        .or_else(|| ts.strip_suffix(" calldata"))
        .or_else(|| ts.strip_suffix(" storage ref"))
        .or_else(|| ts.strip_suffix(" storage pointer"))
        .or_else(|| ts.strip_suffix(" memory ref"))
        .unwrap_or(ts);

    // Dynamic arrays: any type containing "[]".
    if base.contains("[]") {
        return true;
    }

    // Bare `bytes` (not bytes1..bytes32): exactly "bytes" after stripping location.
    if base == "bytes" {
        return true;
    }

    // `string` type.
    if base == "string" {
        return true;
    }

    false
}

impl IssueDetector for AbiEncodePackedHashCollisionDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for call in context.function_calls() {
            // The call expression must be a MemberAccess with member_name == "encodePacked"
            // whose base is the identifier "abi".
            if let Expression::MemberAccess(ma) = call.expression.as_ref() {
                if ma.member_name != "encodePacked" {
                    continue;
                }
                if let Expression::Identifier(base_id) = ma.expression.as_ref() {
                    if base_id.name != "abi" {
                        continue;
                    }
                } else {
                    continue;
                }
            } else {
                continue;
            }

            // Count how many arguments have a dynamic type.
            let dynamic_count = call
                .arguments
                .iter()
                .filter(|arg| is_dynamic_type(arg.type_string()))
                .count();

            if dynamic_count >= 2 {
                capture!(self, context, call);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Hash Collision with abi.encodePacked".to_string()
    }

    fn description(&self) -> String {
        "Passing two or more dynamic types (string, bytes, or arrays) to abi.encodePacked can \
         produce the same byte sequence for different inputs, allowing an attacker to forge \
         signatures or bypass hash-based access controls; use abi.encode instead, which \
         length-prefixes each value and eliminates the ambiguity."
            .to_string()
    }

    fn name(&self) -> String {
        "abi-encode-packed-hash-collision".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
