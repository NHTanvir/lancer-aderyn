//! `incorrect-shift-in-assembly` — flags shift operations where a number literal is shifted by a variable.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct IncorrectShiftOrderDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for IncorrectShiftOrderDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for binop in context.binary_operations() {
            if !matches!(binop.operator.as_str(), "<<" | ">>") {
                continue;
            }
            // A hit: the left operand is a number Literal AND the right operand is NOT a Literal.
            // This matches `2 << x` — likely the arguments are reversed; the intent was `x << 2`.
            let left_is_number_literal = matches!(
                binop.left_expression.as_ref(),
                Expression::Literal(lit) if lit.kind.as_deref() == Some("number")
            );
            let right_is_literal = matches!(
                binop.right_expression.as_ref(),
                Expression::Literal(_)
            );
            if left_is_number_literal && !right_is_literal {
                capture!(self, context, binop);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Incorrect Shift Operand Order".to_string()
    }

    fn description(&self) -> String {
        "The shift expression has a numeric literal on the left-hand side and a variable on the \
         right-hand side (e.g. `2 << x`), which almost certainly has the operands reversed; \
         rewrite it as `x << 2` (or the equivalent right-shift) so that the value is shifted \
         by the constant, not the constant shifted by the value."
            .to_string()
    }

    fn name(&self) -> String {
        "incorrect-shift-in-assembly".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
