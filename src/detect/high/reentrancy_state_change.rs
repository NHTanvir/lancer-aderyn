//! `reentrancy-state-change` — flags state changes that follow an external call (classic reentrancy pattern).

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::browser::ApproximateStorageChangeFinder;
use crate::context::flow;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::{has_modifier_like, is_view_or_pure};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct ReentrancyStateChangeDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// External-call member names that transfer control outside this contract.
const EXTERNAL_CALL_MEMBERS: &[&str] = &["call", "delegatecall", "transfer", "send"];

/// Collect only the expressions that belong DIRECTLY to `stmt` — without
/// descending into nested child statements (those are visited separately by the
/// caller's flattened statement list, preserving textual ordering).
fn direct_expressions(stmt: &Statement, out: &mut Vec<Expression>) {
    match stmt {
        Statement::ExpressionStatement(e) => flow::walk_expression(&e.expression, out),
        Statement::VariableDeclarationStatement(v) => {
            if let Some(iv) = &v.initial_value {
                flow::walk_expression(iv, out);
            }
        }
        Statement::Return(r) => {
            if let Some(e) = &r.expression {
                flow::walk_expression(e, out);
            }
        }
        Statement::IfStatement(i) => flow::walk_expression(&i.condition, out),
        Statement::WhileStatement(w) => flow::walk_expression(&w.condition, out),
        Statement::DoWhileStatement(d) => flow::walk_expression(&d.condition, out),
        Statement::ForStatement(f) => {
            if let Some(c) = &f.condition {
                flow::walk_expression(c, out);
            }
        }
        Statement::EmitStatement(e) => {
            flow::walk_expression(&Expression::FunctionCall(e.event_call.clone()), out)
        }
        Statement::RevertStatement(r) => {
            flow::walk_expression(&Expression::FunctionCall(r.error_call.clone()), out)
        }
        _ => {}
    }
}

/// Returns the first DIRECT external call expression in `stmt`, or `None`.
fn find_external_call_in_stmt(stmt: &Statement) -> Option<Expression> {
    let mut exprs = Vec::new();
    direct_expressions(stmt, &mut exprs);
    for expr in exprs {
        if let Some(call_expr) = is_external_call_expr(&expr) {
            return Some(call_expr);
        }
    }
    None
}

/// If `expr` is an external call expression, return the top-level call expression
/// (FunctionCall wrapping the MemberAccess, or the FunctionCallOptions itself).
/// We walk the expression tree looking for the pattern.
fn is_external_call_expr(expr: &Expression) -> Option<Expression> {
    match expr {
        // addr.call(...), addr.send(...), addr.transfer(...), addr.delegatecall(...)
        Expression::FunctionCall(fc) => {
            if let Expression::MemberAccess(ma) = fc.expression.as_ref() {
                if EXTERNAL_CALL_MEMBERS.contains(&ma.member_name.as_str()) {
                    return Some(expr.clone());
                }
            }
            // Also check FunctionCallOptions nested inside FunctionCall
            // e.g. addr.call{value: v}(data)
            if let Expression::FunctionCallOptions(fco) = fc.expression.as_ref() {
                if fco.names.iter().any(|n| n == "value") {
                    return Some(expr.clone());
                }
                // Underlying member access inside FunctionCallOptions
                if let Expression::MemberAccess(ma) = fco.expression.as_ref() {
                    if EXTERNAL_CALL_MEMBERS.contains(&ma.member_name.as_str()) {
                        return Some(expr.clone());
                    }
                }
            }
            None
        }
        // addr.call{value: v} used as a standalone expression (less common, but correct)
        Expression::FunctionCallOptions(fco) => {
            if fco.names.iter().any(|n| n == "value") {
                return Some(expr.clone());
            }
            if let Expression::MemberAccess(ma) = fco.expression.as_ref() {
                if EXTERNAL_CALL_MEMBERS.contains(&ma.member_name.as_str()) {
                    return Some(expr.clone());
                }
            }
            None
        }
        _ => None,
    }
}

impl IssueDetector for ReentrancyStateChangeDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        let finder = ApproximateStorageChangeFinder::new(context);

        for func in context.function_definitions() {
            // Only check functions that have a body.
            let body = match &func.body {
                Some(b) => b,
                None => continue,
            };

            // Skip view/pure — they cannot write state anyway.
            if is_view_or_pure(func) {
                continue;
            }

            // Skip functions protected by a nonReentrant-like modifier.
            if has_modifier_like(func, "nonreentrant") {
                continue;
            }

            // Flatten the body into an ordered list of statements.
            let mut stmts: Vec<Statement> = Vec::new();
            flow::flatten_statements(&Statement::Block(body.clone()), &mut stmts);

            // Find the index of the first statement that contains an external call.
            let mut first_call_idx: Option<usize> = None;
            let mut call_expr_to_capture: Option<Expression> = None;

            for (idx, stmt) in stmts.iter().enumerate() {
                if let Some(call_expr) = find_external_call_in_stmt(stmt) {
                    first_call_idx = Some(idx);
                    call_expr_to_capture = Some(call_expr);
                    break;
                }
            }

            let first_call_idx = match first_call_idx {
                Some(i) => i,
                None => continue, // No external call — nothing to flag.
            };

            // Check if ANY statement after the first external call mutates state.
            let has_later_state_change = stmts[first_call_idx + 1..]
                .iter()
                .any(|stmt| finder.statement_mutates_state(stmt));

            if has_later_state_change {
                if let Some(call_expr) = call_expr_to_capture {
                    capture!(self, context, &call_expr);
                }
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "State Change After External Call (Reentrancy)".to_string()
    }

    fn description(&self) -> String {
        "Apply the Checks-Effects-Interactions pattern: perform all state changes before making \
         external calls, or use a reentrancy guard modifier (e.g. OpenZeppelin's `nonReentrant`) \
         to prevent recursive reentry that could exploit stale contract state."
            .to_string()
    }

    fn name(&self) -> String {
        "reentrancy-state-change".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
