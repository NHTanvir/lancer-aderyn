//! `block-timestamp-deadline` — flags comparisons that use block.timestamp or now as a deadline.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct BlockTimestampDeadlineDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if the expression is `block.timestamp` (MemberAccess) or the
/// legacy `now` alias (Identifier).
fn is_block_timestamp(expr: &Expression) -> bool {
    match expr {
        Expression::MemberAccess(ma) => {
            if ma.member_name != "timestamp" {
                return false;
            }
            matches!(ma.expression.as_ref(), Expression::Identifier(id) if id.name == "block")
        }
        Expression::Identifier(id) => id.name == "now",
        _ => false,
    }
}

impl IssueDetector for BlockTimestampDeadlineDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        const DEADLINE_OPS: &[&str] = &["<", "<=", ">", ">="];

        for binop in context.binary_operations() {
            if !DEADLINE_OPS.contains(&binop.operator.as_str()) {
                continue;
            }
            if is_block_timestamp(binop.left_expression.as_ref())
                || is_block_timestamp(binop.right_expression.as_ref())
            {
                capture!(self, context, binop);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "block.timestamp Used as Deadline".to_string()
    }

    fn description(&self) -> String {
        "Using block.timestamp (or the deprecated now alias) as a transaction deadline allows \
         validators to manipulate block timestamps within the ~15-second drift window and \
         arbitrarily satisfy or reject the deadline check; pass an explicit caller-supplied \
         deadline parameter instead and validate it against block.timestamp only at call time."
            .to_string()
    }

    fn name(&self) -> String {
        "block-timestamp-deadline".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
