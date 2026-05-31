//! `assert-state-change` — flags state-mutating expressions inside assert().

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::browser::ApproximateStorageChangeFinder;
use crate::context::flow;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct AssertStateChangeDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if the expression tree rooted at `expr` contains any
/// assignment or increment/decrement unary operation (++/--).
fn contains_mutating_subexpr(expr: &Expression) -> bool {
    let mut exprs: Vec<Expression> = Vec::new();
    flow::walk_expression(expr, &mut exprs);
    for e in &exprs {
        match e {
            Expression::Assignment(_) => return true,
            Expression::UnaryOperation(u) => {
                let op = u.operator.as_str();
                if op == "++" || op == "--" {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

impl IssueDetector for AssertStateChangeDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        let finder = ApproximateStorageChangeFinder::new(context);

        for call in context.function_calls() {
            // Must be a plain `assert(...)` identifier call with exactly 1 argument.
            if let Expression::Identifier(id) = call.expression.as_ref() {
                if id.name != "assert" {
                    continue;
                }
            } else {
                continue;
            }

            if call.arguments.len() != 1 {
                continue;
            }

            let arg = &call.arguments[0];

            // Flag if the argument expression itself mutates storage state, OR
            // if it contains any syntactic assignment / ++ / -- subexpression.
            if finder.expression_mutates_state(arg) || contains_mutating_subexpr(arg) {
                capture!(self, context, call);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "State Change in assert()".to_string()
    }

    fn description(&self) -> String {
        "The argument to assert() performs a state-mutating operation (assignment, increment, or \
         decrement); because assert() is intended only for invariant checking and its argument may \
         be optimized away or stripped by the compiler in certain build modes, move all state \
         changes out of assert() and assert only on the resulting value."
            .to_string()
    }

    fn name(&self) -> String {
        "assert-state-change".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
