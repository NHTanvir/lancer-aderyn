//! `unchecked-low-level-call` — flags low-level calls whose return value is discarded.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::is_low_level_call_member;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct UncheckedLowLevelCallDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if `expr` is a member access targeting a low-level call member
/// (`call`, `delegatecall`, or `staticcall`), possibly with a `FunctionCallOptions`
/// wrapper (e.g. `addr.call{value: x}` before the argument list is applied).
fn expression_is_low_level_call(expr: &Expression) -> bool {
    match expr {
        // Plain: addr.call(...) — expression is MemberAccess
        Expression::MemberAccess(ma) => is_low_level_call_member(&ma.member_name),
        // Options: addr.call{value: x}(...) — expression is FunctionCallOptions
        // whose inner expression is the MemberAccess
        Expression::FunctionCallOptions(fco) => match fco.expression.as_ref() {
            Expression::MemberAccess(ma) => is_low_level_call_member(&ma.member_name),
            _ => false,
        },
        _ => false,
    }
}

impl IssueDetector for UncheckedLowLevelCallDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Iterate all FunctionCall nodes.
        // A hit requires two conditions:
        //   1. The call expression resolves to a low-level call member
        //      (possibly through a FunctionCallOptions wrapper).
        //   2. The direct parent is an ExpressionStatement, meaning the
        //      (bool, bytes) return tuple is thrown away entirely.
        for call in context.function_calls() {
            if !expression_is_low_level_call(call.expression.as_ref()) {
                continue;
            }

            // Check the immediate parent node.
            match context.get_parent(call.id) {
                Some(AstNode::ExpressionStatement(_)) => {
                    // Return value is discarded — flag it.
                    capture!(self, context, call);
                }
                _ => {
                    // Parent is an Assignment, VariableDeclarationStatement,
                    // IfStatement condition, require argument, etc. — result is used.
                }
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Unchecked Low-level Call".to_string()
    }

    fn description(&self) -> String {
        "The return value of a low-level call (`call`, `delegatecall`, or `staticcall`) is not \
         checked; always capture the boolean success flag and revert on failure, or use \
         OpenZeppelin's `Address.functionCall` / `sendValue` wrappers that revert automatically."
            .to_string()
    }

    fn name(&self) -> String {
        "unchecked-low-level-call".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
