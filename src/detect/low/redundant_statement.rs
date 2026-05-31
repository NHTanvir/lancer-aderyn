//! `redundant-statements` — flags expression statements that have no effect.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct RedundantStatementDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if the expression is a pure identifier (no side effects).
fn is_pure_identifier(expr: &Expression) -> bool {
    matches!(expr, Expression::Identifier(_))
}

/// Returns true when the expression has no observable effect:
///   - a bare Identifier (e.g. `myVar;`)
///   - a bare Literal (e.g. `42;`, `"hello";`, `true;`)
///   - an ElementaryTypeNameExpression (e.g. `uint256;`)
///   - a Conditional whose true- and false-branches are both pure identifiers
///     (e.g. `flag ? a : b;` where neither branch has side effects)
fn is_no_effect_expression(expr: &Expression) -> bool {
    match expr {
        Expression::Identifier(_) => true,
        Expression::Literal(_) => true,
        Expression::ElementaryTypeNameExpression(_) => true,
        Expression::Conditional(cond) => {
            is_pure_identifier(cond.true_expression.as_ref())
                && is_pure_identifier(cond.false_expression.as_ref())
        }
        _ => false,
    }
}

impl IssueDetector for RedundantStatementDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for stmt in context.expression_statements() {
            if is_no_effect_expression(&stmt.expression) {
                capture!(self, context, stmt);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Redundant Statement".to_string()
    }

    fn description(&self) -> String {
        "Remove the statement — a bare identifier, literal, type name, or value-only conditional \
         expression used as a statement has no effect and is dead code that misleads readers."
            .to_string()
    }

    fn name(&self) -> String {
        "redundant-statements".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
