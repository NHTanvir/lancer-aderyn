//! `unchecked-return` — a function call that returns a value used as a bare
//! statement (the return value is silently discarded).

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct UncheckedReturnDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for UncheckedReturnDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for stmt in context.expression_statements() {
            let Expression::FunctionCall(call) = &stmt.expression else {
                continue;
            };
            // Only ordinary calls (not casts / struct constructors).
            if call.kind.as_deref() != Some("functionCall") {
                continue;
            }
            // Must return something.
            let ret = call.type_descriptions.type_string.as_deref().unwrap_or("");
            if ret.is_empty() || ret == "tuple()" || ret == "()" {
                continue;
            }
            // Exclude calls handled by dedicated detectors / control-flow builtins.
            let excluded = match call.expression.as_ref() {
                Expression::Identifier(id) => {
                    matches!(id.name.as_str(), "require" | "assert" | "revert")
                }
                Expression::MemberAccess(m) => matches!(
                    m.member_name.as_str(),
                    "call" | "delegatecall" | "staticcall" | "send" | "transfer"
                ),
                _ => false,
            };
            if excluded {
                continue;
            }
            capture!(self, context, call);
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Unchecked Return Value".to_string()
    }

    fn description(&self) -> String {
        "This call returns a value that is silently discarded. If the return value signals success \
         or carries data, check it; otherwise document why it is intentionally ignored."
            .to_string()
    }

    fn name(&self) -> String {
        "unchecked-return".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
