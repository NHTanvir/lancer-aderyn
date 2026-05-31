//! `misused-boolean` — flags binary expressions that compare or combine a boolean literal redundantly.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct MisusedBooleanDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if the given expression is a boolean literal (`true` or `false`).
fn is_bool_literal(expr: &Expression) -> bool {
    if let Expression::Literal(lit) = expr {
        return lit.kind.as_deref() == Some("bool");
    }
    false
}

impl IssueDetector for MisusedBooleanDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for binop in context.binary_operations() {
            let op = binop.operator.as_str();
            let left = binop.left_expression.as_ref();
            let right = binop.right_expression.as_ref();

            match op {
                // `expr && true`, `false || expr`, etc. — one side is a bool literal,
                // making the whole expression trivially reducible.
                "&&" | "||" => {
                    if is_bool_literal(left) || is_bool_literal(right) {
                        capture!(self, context, binop);
                    }
                }
                // `expr == true`, `expr != false`, etc. — comparing a bool to a literal
                // is always redundant; use the expression directly or negate it.
                "==" | "!=" => {
                    if is_bool_literal(left) || is_bool_literal(right) {
                        capture!(self, context, binop);
                    }
                }
                _ => {}
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Misused Boolean Operator".to_string()
    }

    fn description(&self) -> String {
        "Comparing or combining a boolean expression with a boolean literal (`true`/`false`) is \
         always redundant and can signal a logic error; use the boolean expression directly, or \
         negate it with `!`, and remove the literal operand."
            .to_string()
    }

    fn name(&self) -> String {
        "misused-boolean".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
