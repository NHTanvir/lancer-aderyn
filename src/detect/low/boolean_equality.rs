//! `boolean-equality` — flags explicit comparisons of a boolean expression to a bool literal.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct BooleanEqualityDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true when the expression is a bool literal (`true` or `false`).
fn is_bool_literal(expr: &Expression) -> bool {
    if let Expression::Literal(lit) = expr {
        return lit.kind.as_deref() == Some("bool");
    }
    false
}

impl IssueDetector for BooleanEqualityDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for binop in context.binary_operations() {
            if binop.operator != "==" && binop.operator != "!=" {
                continue;
            }
            let lhs = binop.left_expression.as_ref();
            let rhs = binop.right_expression.as_ref();
            if is_bool_literal(lhs) || is_bool_literal(rhs) {
                capture!(self, context, binop);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Boolean Equality (== true / == false)".to_string()
    }

    fn description(&self) -> String {
        "Comparing a boolean expression to a bool literal with `==` or `!=` is redundant; \
         use the boolean expression directly (or negate it with `!`) to improve readability \
         and avoid confusion."
            .to_string()
    }

    fn name(&self) -> String {
        "boolean-equality".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
