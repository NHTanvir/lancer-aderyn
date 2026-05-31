//! `incorrect-use-of-caret-operator` — flags `^` used where `**` (exponentiation) was likely intended.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct IncorrectCaretOperatorDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if `expr` is a number Literal.
fn is_number_literal(expr: &Expression) -> bool {
    if let Expression::Literal(lit) = expr {
        lit.kind.as_deref() == Some("number")
    } else {
        false
    }
}

impl IssueDetector for IncorrectCaretOperatorDetector {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for binop in ctx.binary_operations() {
            if binop.operator != "^" {
                continue;
            }
            // The result type must be an integer type.
            let type_str = binop
                .common_type
                .type_string
                .as_deref()
                .unwrap_or("");
            if !type_str.contains("int") {
                continue;
            }
            // At least one operand must be a number literal — strong signal that the
            // developer wrote `base ^ exp` intending exponentiation rather than XOR.
            if is_number_literal(&binop.left_expression)
                || is_number_literal(&binop.right_expression)
            {
                capture!(self, ctx, binop);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Caret (^) Used as Exponentiation".to_string()
    }

    fn description(&self) -> String {
        "In Solidity, `^` is the bitwise XOR operator, not exponentiation; replace `a ^ b` with \
         `a ** b` to perform exponentiation, or use a well-audited math library such as \
         OpenZeppelin's `Math.pow` to avoid silent logic errors."
            .to_string()
    }

    fn name(&self) -> String {
        "incorrect-use-of-caret-operator".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
