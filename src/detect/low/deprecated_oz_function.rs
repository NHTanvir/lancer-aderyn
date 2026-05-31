//! `deprecated-oz-functions` — flags calls to deprecated OpenZeppelin functions.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

/// The set of OpenZeppelin function names that have been deprecated.
const DEPRECATED_OZ_FUNCTIONS: &[&str] = &[
    "_setupRole",
    "safeApprove",
    "functionCallWithValue",
    "setupRole",
    "_msgData",
];

#[derive(Default)]
pub struct DeprecatedOzFunctionDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns the callee name from a function call expression, if it is a
/// `MemberAccess` (e.g. `token.safeApprove(...)`) or a bare `Identifier`
/// (e.g. `_setupRole(...)`).
fn callee_name(expr: &Expression) -> Option<&str> {
    match expr {
        Expression::MemberAccess(ma) => Some(ma.member_name.as_str()),
        Expression::Identifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

impl IssueDetector for DeprecatedOzFunctionDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for call in context.function_calls() {
            if let Some(name) = callee_name(call.expression.as_ref()) {
                if DEPRECATED_OZ_FUNCTIONS.contains(&name) {
                    capture!(self, context, call);
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Deprecated OpenZeppelin Function".to_string()
    }

    fn description(&self) -> String {
        "Replace deprecated OpenZeppelin functions (`_setupRole`, `safeApprove`, \
         `functionCallWithValue`, `setupRole`, `_msgData`) with their current equivalents \
         (`_grantRole`, `forceApprove`/`safeIncreaseAllowance`, `functionCallWithValue` from \
         Address, `grantRole`, and `msg.data` respectively) to avoid relying on removed or \
         insecure API surface in future library upgrades."
            .to_string()
    }

    fn name(&self) -> String {
        "deprecated-oz-functions".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
