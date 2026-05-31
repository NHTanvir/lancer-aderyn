//! `delegate-call-in-unprotected-public-function` — flags delegatecall to an unchecked address.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct DelegateCallUncheckedAddressDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if the expression is `address(this)` — a type-conversion call
/// `address(this)` has kind "typeConversion" and a single `this` Identifier arg.
fn is_address_this(expr: &Expression) -> bool {
    if let Expression::FunctionCall(call) = expr {
        if call.kind.as_deref() == Some("typeConversion") {
            if let Expression::ElementaryTypeNameExpression(_) = call.expression.as_ref() {
                if call.arguments.len() == 1 {
                    if let Expression::Identifier(id) = &call.arguments[0] {
                        return id.name == "this";
                    }
                }
            }
        }
    }
    false
}

/// Returns true if the expression is the bare `this` keyword (Identifier named "this").
fn is_this(expr: &Expression) -> bool {
    if let Expression::Identifier(id) = expr {
        return id.name == "this";
    }
    false
}

/// Returns true when the base of `.delegatecall(...)` looks like a safe, known
/// constant — specifically `address(this)` or `this`.  Everything else (an
/// arbitrary Identifier, a state-variable, a function parameter, etc.) is
/// treated as potentially dangerous.
fn base_is_safe(expr: &Expression) -> bool {
    is_address_this(expr) || is_this(expr)
}

impl IssueDetector for DelegateCallUncheckedAddressDetector {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for ma in ctx.member_accesses() {
            if ma.member_name != "delegatecall" {
                continue;
            }
            // Capture every delegatecall whose base is NOT address(this) / this.
            if !base_is_safe(ma.expression.as_ref()) {
                capture!(self, ctx, ma);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Delegatecall to Unchecked Address".to_string()
    }

    fn description(&self) -> String {
        "Validate that the target address passed to `delegatecall` is a trusted, \
         immutable contract address (e.g. stored as an `immutable` variable set in \
         the constructor) — never allow an arbitrary caller-supplied or mutable \
         storage address, as a malicious target can overwrite the proxy's storage \
         and take full control of the contract."
            .to_string()
    }

    fn name(&self) -> String {
        "delegate-call-in-unprotected-public-function".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
