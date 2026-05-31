//! `division-before-multiplication` — flags multiplication whose operand is a division result.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct DivisionBeforeMultiplicationDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Unwrap a single-component, non-inline-array TupleExpression to its inner
/// Expression, recursively, so that `(a / b) * c` is treated the same as
/// `a / b * c`.
fn unwrap_tuple(expr: &Expression) -> &Expression {
    if let Expression::TupleExpression(tuple) = expr {
        if !tuple.is_inline_array && tuple.components.len() == 1 {
            if let Some(Some(inner)) = tuple.components.first() {
                return unwrap_tuple(inner);
            }
        }
    }
    expr
}

/// Return true if `expr` is a BinaryOperation with operator `"/"` (after
/// unwrapping any surrounding parentheses / single-component tuples).
fn is_division(expr: &Expression) -> bool {
    match unwrap_tuple(expr) {
        Expression::BinaryOperation(binop) => binop.operator == "/",
        _ => false,
    }
}

impl IssueDetector for DivisionBeforeMultiplicationDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for binop in context.binary_operations() {
            if binop.operator != "*" {
                continue;
            }
            // Flag if either operand is (or unwraps to) a division operation.
            if is_division(binop.left_expression.as_ref())
                || is_division(binop.right_expression.as_ref())
            {
                capture!(self, context, binop);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Division Before Multiplication".to_string()
    }

    fn description(&self) -> String {
        "Performing division before multiplication can silently truncate intermediate results due \
         to integer rounding; reorder operations so that all multiplications are carried out first \
         and division is applied last to preserve precision."
            .to_string()
    }

    fn name(&self) -> String {
        "division-before-multiplication".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
