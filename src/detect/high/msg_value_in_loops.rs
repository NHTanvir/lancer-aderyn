//! `msg-value-in-loop` — flags use of msg.value inside a loop body.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::is_inside_loop;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct MsgValueInLoopsDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for MsgValueInLoopsDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for m in context.member_accesses() {
            if m.member_name == "value" {
                if let Expression::Identifier(base) = m.expression.as_ref() {
                    if base.name == "msg" && is_inside_loop(context, m.id) {
                        capture!(self, context, m);
                    }
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "msg.value Used Inside a Loop".to_string()
    }

    fn description(&self) -> String {
        "Using msg.value inside a loop is dangerous because msg.value is fixed for the entire \
         transaction, so each iteration reuses the same value rather than receiving independent \
         Ether amounts; refactor the logic to accept per-iteration amounts via function parameters \
         or accumulate a single payment outside the loop."
            .to_string()
    }

    fn name(&self) -> String {
        "msg-value-in-loop".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
