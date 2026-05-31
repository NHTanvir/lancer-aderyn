//! `delegate-call-in-loop` — flags `delegatecall` invocations inside loops.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::is_inside_loop;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct DelegatecallInLoopDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for DelegatecallInLoopDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for m in context.member_accesses() {
            if m.member_name == "delegatecall" && is_inside_loop(context, m.id) {
                capture!(self, context, m);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Delegatecall Inside a Loop".to_string()
    }

    fn description(&self) -> String {
        "Avoid using `delegatecall` inside a loop; if any iteration reverts or an attacker \
         controls the callee, it can corrupt contract storage or cause a denial-of-service — \
         refactor to call the target once outside the loop or use a pattern that does not \
         rely on delegatecall for repeated iteration."
            .to_string()
    }

    fn name(&self) -> String {
        "delegate-call-in-loop".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
