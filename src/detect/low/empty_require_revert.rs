//! `require-with-string` — flags require/revert calls that omit a descriptive message.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct EmptyRequireRevertDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if the expression is a string Literal with an empty or absent value.
fn is_empty_string_literal(expr: &Expression) -> bool {
    if let Expression::Literal(lit) = expr {
        if lit.kind.as_deref() == Some("string") {
            return lit.value.as_deref().map(|v| v.is_empty()).unwrap_or(true);
        }
    }
    false
}

impl IssueDetector for EmptyRequireRevertDetector {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for call in ctx.function_calls() {
            match call.expression.as_ref() {
                Expression::Identifier(id) if id.name == "require" => {
                    // require(condition) — no message at all (1 arg)
                    if call.arguments.len() == 1 {
                        capture!(self, ctx, call);
                    }
                    // require(condition, "") — empty string message (2 args, second is "")
                    else if call.arguments.len() == 2
                        && is_empty_string_literal(&call.arguments[1])
                    {
                        capture!(self, ctx, call);
                    }
                }
                Expression::Identifier(id) if id.name == "revert" => {
                    // revert() — no arguments
                    if call.arguments.is_empty() {
                        capture!(self, ctx, call);
                    }
                    // revert("") — single empty string argument
                    else if call.arguments.len() == 1
                        && is_empty_string_literal(&call.arguments[0])
                    {
                        capture!(self, ctx, call);
                    }
                }
                _ => {}
            }
        }

        // Also catch RevertStatement nodes where the error_call has no arguments
        // (covers `revert SomeError()` with zero fields, and plain `revert()`).
        for rev in ctx.revert_statements() {
            let error_call = &rev.error_call;
            // Only flag calls whose expression resolves to a bare identifier
            // (plain revert or revert Error(...) with no message args).
            if error_call.arguments.is_empty() {
                // Distinguish `revert()` (Identifier) from `revert CustomError()` (Identifier to
                // a user-defined error) — flag both as lacking a human-readable string.
                capture!(self, ctx, error_call);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Empty Require/Revert Message".to_string()
    }

    fn description(&self) -> String {
        "Provide a descriptive string message to every require() and revert() so that callers and \
         off-chain tooling can understand why the transaction was rejected without inspecting \
         bytecode."
            .to_string()
    }

    fn name(&self) -> String {
        "require-with-string".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
