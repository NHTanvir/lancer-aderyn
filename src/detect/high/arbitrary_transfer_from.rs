//! `arbitrary-transfer-from` — flags transferFrom/safeTransferFrom calls where the `from`
//! argument is not `msg.sender` or `address(this)`, allowing an attacker to drain arbitrary accounts.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct ArbitraryTransferFromDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if `expr` is `msg.sender`.
fn is_msg_sender(expr: &Expression) -> bool {
    if let Expression::MemberAccess(ma) = expr {
        if ma.member_name == "sender" {
            if let Expression::Identifier(base) = ma.expression.as_ref() {
                return base.name == "msg";
            }
        }
    }
    false
}

/// Returns true if `expr` is `address(this)`.
/// Solc emits this as a typeConversion FunctionCall: address(<Identifier "this"> or MemberAccess "this").
fn is_address_this(expr: &Expression) -> bool {
    if let Expression::FunctionCall(call) = expr {
        // Must be a type-conversion call, e.g. address(this)
        if call.kind.as_deref() == Some("typeConversion") {
            if let Expression::ElementaryTypeNameExpression(_) = call.expression.as_ref() {
                // Check the single argument is `this`
                if let Some(arg) = call.arguments.first() {
                    return is_this(arg);
                }
            }
        }
    }
    false
}

/// Returns true if `expr` refers to the special `this` keyword (an Identifier named "this"
/// or a MemberAccess whose member_name is "this").
fn is_this(expr: &Expression) -> bool {
    match expr {
        Expression::Identifier(id) => id.name == "this",
        Expression::MemberAccess(ma) => ma.member_name == "this",
        _ => false,
    }
}

/// Returns true if `expr` is a safe/known `from` value (msg.sender or address(this)).
fn is_safe_from(expr: &Expression) -> bool {
    is_msg_sender(expr) || is_address_this(expr)
}

impl IssueDetector for ArbitraryTransferFromDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for call in context.function_calls() {
            // The function being called must be a MemberAccess with member_name
            // "transferFrom" or "safeTransferFrom".
            let ma = match call.expression.as_ref() {
                Expression::MemberAccess(ma) => ma,
                _ => continue,
            };

            if ma.member_name != "transferFrom" && ma.member_name != "safeTransferFrom" {
                continue;
            }

            // There must be at least one argument (the `from` address).
            let from_arg = match call.arguments.first() {
                Some(a) => a,
                None => continue,
            };

            // Flag the call unless `from` is provably msg.sender or address(this).
            if !is_safe_from(from_arg) {
                capture!(self, context, call);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Arbitrary `from` in transferFrom".to_string()
    }

    fn description(&self) -> String {
        "Passing a caller-controlled address as the `from` argument to `transferFrom` or \
         `safeTransferFrom` allows an attacker to transfer tokens from any account that has \
         approved the contract, draining funds they do not own. Ensure the `from` argument \
         is always `msg.sender` or a value that the contract itself controls."
            .to_string()
    }

    fn name(&self) -> String {
        "arbitrary-transfer-from".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
