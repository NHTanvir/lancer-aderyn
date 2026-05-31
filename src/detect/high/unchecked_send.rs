//! `unchecked-send` — flags `.send()` calls whose boolean return value is ignored.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct UncheckedSendDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for UncheckedSendDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for call in context.function_calls() {
            // Check that the callee is a MemberAccess with member_name == "send".
            if let Expression::MemberAccess(ma) = call.expression.as_ref() {
                if ma.member_name != "send" {
                    continue;
                }
            } else {
                continue;
            }

            // The return value is ignored when the immediate parent is an
            // ExpressionStatement (i.e. the call result is not passed to
            // require/if/assignment — it is simply discarded).
            if let Some(parent) = context.get_parent(call.id) {
                if parent.node_type() == "ExpressionStatement" {
                    capture!(self, context, call);
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Unchecked `send`".to_string()
    }

    fn description(&self) -> String {
        "The return value of `address.send()` is a boolean indicating success; ignoring it means \
         a failed Ether transfer goes undetected — always check the return value or use \
         `Address.sendValue` from OpenZeppelin's `Address` library, which reverts on failure."
            .to_string()
    }

    fn name(&self) -> String {
        "unchecked-send".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
