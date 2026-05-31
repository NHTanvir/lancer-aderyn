//! `unsafe-casting` — flags narrowing integer casts that silently truncate bits.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct UnsafeCastingDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Parse `"uint<N>"` / `"int<N>"` -> bit-width.
///
/// * `"uint"` and `"int"` (no suffix) are treated as 256 bits.
/// * Returns `None` for any other type string.
fn integer_bits(type_str: &str) -> Option<u32> {
    let s = type_str.trim();
    if s == "uint" || s == "int" {
        return Some(256);
    }
    if let Some(rest) = s.strip_prefix("uint") {
        return rest.parse::<u32>().ok();
    }
    if let Some(rest) = s.strip_prefix("int") {
        return rest.parse::<u32>().ok();
    }
    None
}

/// True when both type strings name integer types and the target is narrower
/// than the source (a potentially lossy downcast).
fn is_narrowing_cast(target_type: &str, source_type: &str) -> bool {
    if let (Some(target_bits), Some(source_bits)) =
        (integer_bits(target_type), integer_bits(source_type))
    {
        return target_bits < source_bits;
    }
    false
}

impl IssueDetector for UnsafeCastingDetector {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for call in ctx.function_calls() {
            // Only interested in explicit type-conversion calls.
            if call.kind.as_deref() != Some("typeConversion") {
                continue;
            }
            // Must have exactly one argument.
            if call.arguments.len() != 1 {
                continue;
            }

            // The target type is the type_descriptions of the FunctionCall itself.
            let target_type = match call.type_descriptions.type_string.as_deref() {
                Some(t) => t,
                None => continue,
            };

            // The source type is the type_string of the sole argument.
            let source_type = match call.arguments[0].type_string() {
                Some(t) => t,
                None => continue,
            };

            if is_narrowing_cast(target_type, source_type) {
                capture!(self, ctx, call);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Unsafe Downcasting".to_string()
    }

    fn description(&self) -> String {
        "Casting a wider integer type to a narrower one (e.g. uint256 to uint128) silently \
         truncates the high-order bits and can produce an incorrect value with no revert; \
         use OpenZeppelin's SafeCast library or add an explicit bounds check before the cast."
            .to_string()
    }

    fn name(&self) -> String {
        "unsafe-casting".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
