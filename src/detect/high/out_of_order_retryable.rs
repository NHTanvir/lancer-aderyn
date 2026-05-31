//! `out-of-order-retryable` — flags functions that create multiple retryable tickets out of order.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::is_inside_loop;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct OutOfOrderRetryableDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Return true if the function call is a `createRetryableTicket` or
/// `unsafeCreateRetryableTicket` invocation (member-access or bare identifier).
fn is_retryable_ticket_call(call: &FunctionCall) -> bool {
    match call.expression.as_ref() {
        Expression::MemberAccess(ma) => {
            matches!(
                ma.member_name.as_str(),
                "createRetryableTicket" | "unsafeCreateRetryableTicket"
            )
        }
        Expression::Identifier(id) => {
            matches!(
                id.name.as_str(),
                "createRetryableTicket" | "unsafeCreateRetryableTicket"
            )
        }
        _ => false,
    }
}

impl IssueDetector for OutOfOrderRetryableDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Group all retryable-ticket calls by the enclosing function id.
        // key = enclosing function NodeID, value = list of &FunctionCall
        let mut calls_per_function: BTreeMap<NodeID, Vec<&FunctionCall>> = BTreeMap::new();

        for call in context.function_calls() {
            if !is_retryable_ticket_call(call) {
                continue;
            }
            if let Some(f) = context.enclosing_function(call.id) {
                calls_per_function.entry(f.id).or_default().push(call);
            }
        }

        for (_func_id, calls) in &calls_per_function {
            let multiple = calls.len() >= 2;
            for call in calls {
                let in_loop = is_inside_loop(context, call.id);
                if multiple || in_loop {
                    capture!(self, context, *call);
                }
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Out-of-order Retryable Tickets".to_string()
    }

    fn description(&self) -> String {
        "Creating multiple retryable tickets in a single function, or creating them inside a loop, \
         can result in out-of-order execution on the L2 because each ticket is an independent \
         cross-chain message with no sequencing guarantee; refactor so that each function issues \
         at most one retryable ticket and avoid loops around ticket creation."
            .to_string()
    }

    fn name(&self) -> String {
        "out-of-order-retryable".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
