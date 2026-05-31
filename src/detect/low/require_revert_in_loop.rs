//! `require-revert-in-loop` — flags `require`/`revert` used inside a loop.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::is_inside_loop;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct RequireRevertInLoopDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for RequireRevertInLoopDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Detect `require(...)` calls inside a loop.
        for call in context.function_calls() {
            if let Expression::Identifier(id) = call.expression.as_ref() {
                if id.name == "require" && is_inside_loop(context, call.id) {
                    capture!(self, context, call);
                }
            }
        }

        // Detect `revert` statements inside a loop.
        for r in context.revert_statements() {
            if is_inside_loop(context, r.id) {
                capture!(self, context, r);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "require/revert Inside a Loop".to_string()
    }

    fn description(&self) -> String {
        "Using `require` or `revert` inside a loop can cause the entire transaction to revert if \
         any single iteration fails; consider accumulating results or using a try/catch pattern so \
         that individual failures are handled gracefully without reverting all prior work."
            .to_string()
    }

    fn name(&self) -> String {
        "require-revert-in-loop".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
