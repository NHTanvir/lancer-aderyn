//! `enumerable-loop-removal` — flags removal or deletion from a collection inside a loop body.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::flow;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct EnumerableLoopRemovalDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Return true if `expr` is a FunctionCall whose callee is a MemberAccess with
/// member_name in {"remove", "pop"}.
fn is_collection_removal_call(expr: &Expression) -> bool {
    if let Expression::FunctionCall(call) = expr {
        if let Expression::MemberAccess(ma) = call.expression.as_ref() {
            if matches!(ma.member_name.as_str(), "remove" | "pop") {
                return true;
            }
        }
    }
    false
}

/// Return true if `expr` is `delete arr[i]` — a UnaryOperation with operator
/// "delete" whose sub-expression is an IndexAccess.
fn is_delete_index(expr: &Expression) -> bool {
    if let Expression::UnaryOperation(u) = expr {
        if u.operator == "delete" {
            if let Expression::IndexAccess(_) = u.sub_expression.as_ref() {
                return true;
            }
        }
    }
    false
}

/// Gather all expressions in the direct body of a loop (the `Box<Statement>`)
/// and check each one for collection-removal patterns.  Captures hits into
/// `self.found_instances`.
fn check_loop_body(
    detector: &mut EnumerableLoopRemovalDetector,
    context: &WorkspaceContext,
    body: &Statement,
) {
    let mut exprs: Vec<Expression> = Vec::new();
    flow::expressions_in_statement(body, &mut exprs);

    for expr in &exprs {
        if is_collection_removal_call(expr) || is_delete_index(expr) {
            capture!(detector, context, expr);
        }
    }
}

impl IssueDetector for EnumerableLoopRemovalDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // for (...) { ... remove/pop/delete ... }
        for stmt in context.for_statements() {
            check_loop_body(self, context, &stmt.body);
        }

        // while (...) { ... remove/pop/delete ... }
        for stmt in context.while_statements() {
            check_loop_body(self, context, &stmt.body);
        }

        // do { ... remove/pop/delete ... } while (...)
        for stmt in context.do_while_statements() {
            check_loop_body(self, context, &stmt.body);
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Modifying Collection While Iterating".to_string()
    }

    fn description(&self) -> String {
        "Calling remove/pop on a dynamic array or mapping, or using delete on an index, \
         inside a loop that iterates the same collection can cause index corruption, \
         skipped elements, or out-of-bounds panics; refactor to collect indices first \
         and apply removals after the loop, or iterate in reverse order."
            .to_string()
    }

    fn name(&self) -> String {
        "enumerable-loop-removal".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
