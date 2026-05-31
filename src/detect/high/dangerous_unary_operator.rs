//! `dangerous-unary-operator` — flags `x =+ y` / `x =- y` / `x =~ y` typos.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct DangerousUnaryOperatorDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for DangerousUnaryOperatorDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Catch the typo pattern `x =- 1` (parsed by solc as `x = (-1)`):
        // the assignment operator is plain `=` and the RHS is a prefix unary
        // operation whose operator is one of `+`, `-`, or `~`.
        for assignment in context.assignments() {
            if assignment.operator != "=" {
                continue;
            }
            if let Expression::UnaryOperation(unary) = assignment.right_hand_side.as_ref() {
                if unary.prefix && matches!(unary.operator.as_str(), "+" | "-" | "~") {
                    capture!(self, context, assignment);
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Dangerous Unary Operator (=+ / =-)".to_string()
    }

    fn description(&self) -> String {
        "Writing `x =- 1` or `x =+ 1` is parsed as `x = (-1)` / `x = (+1)`, not as the \
         compound operators `-=` / `+=`; replace with the intended compound assignment operator \
         (`x -= 1`, `x += 1`) to avoid silent logic errors."
            .to_string()
    }

    fn name(&self) -> String {
        "dangerous-unary-operator".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
