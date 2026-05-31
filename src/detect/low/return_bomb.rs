//! `return-bomb` — flags low-level calls whose returndata is copied into an unbounded `bytes` variable.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct ReturnBombDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if the expression is a low-level call (`call`, `delegatecall`, `staticcall`),
/// possibly wrapped in a `FunctionCallOptions` (e.g. `addr.call{value: v}(...)`).
fn is_low_level_call(expr: &Expression) -> bool {
    match expr {
        // Plain call: `addr.call(...)` — the FunctionCall whose expression is a MemberAccess
        Expression::FunctionCall(fc) => is_low_level_call_expr(fc.expression.as_ref()),
        // Options wrapper: `addr.call{value: v}(...)` — FunctionCall whose expression is
        // FunctionCallOptions whose expression is a MemberAccess
        Expression::FunctionCallOptions(fco) => is_low_level_call_expr(fco.expression.as_ref()),
        _ => false,
    }
}

fn is_low_level_call_expr(expr: &Expression) -> bool {
    match expr {
        Expression::MemberAccess(ma) => {
            matches!(ma.member_name.as_str(), "call" | "delegatecall" | "staticcall")
        }
        // FunctionCallOptions wrapping a MemberAccess
        Expression::FunctionCallOptions(fco) => is_low_level_call_expr(fco.expression.as_ref()),
        _ => false,
    }
}

/// Returns true if any declaration in the statement captures a `bytes`-typed variable.
/// This matches `bytes`, `bytes memory`, `bytes storage`, or any dynamic bytes type.
fn any_decl_is_bytes(decls: &[Option<VariableDeclaration>]) -> bool {
    decls.iter().filter_map(|d| d.as_ref()).any(|decl| {
        // Check type_descriptions first (most reliable)
        if let Some(ts) = decl.type_descriptions.type_string.as_deref() {
            if ts == "bytes" || ts.starts_with("bytes ") {
                return true;
            }
        }
        // Fall back to type_name
        if let Some(tn) = &decl.type_name {
            if let Some(ts) = tn.type_string() {
                if ts == "bytes" || ts.starts_with("bytes ") {
                    return true;
                }
            }
        }
        false
    })
}

impl IssueDetector for ReturnBombDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for stmt in context.variable_declaration_statements() {
            // The statement must have an initial value that is a low-level call.
            let is_llc = stmt
                .initial_value
                .as_ref()
                .map(|iv| is_low_level_call(iv))
                .unwrap_or(false);
            if !is_llc {
                continue;
            }
            // At least one declared variable must capture bytes (unbounded returndata).
            if any_decl_is_bytes(&stmt.declarations) {
                capture!(self, context, stmt);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Return Bomb (Uncapped returndata)".to_string()
    }

    fn description(&self) -> String {
        "Storing the full returndata of a low-level call into a `bytes` variable copies an \
         unbounded amount of data into memory; a malicious callee can return gigabytes of data, \
         causing an out-of-gas revert in the caller. Use assembly with a capped copy length, or \
         verify the return-data size before copying."
            .to_string()
    }

    fn name(&self) -> String {
        "return-bomb".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
