//! `selfdestruct` — flags calls to selfdestruct (or the deprecated alias suicide).

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct SelfdestructDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for SelfdestructDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for call in context.function_calls() {
            if let Expression::Identifier(id) = call.expression.as_ref() {
                if id.name == "selfdestruct" || id.name == "suicide" {
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
        "Use of selfdestruct".to_string()
    }

    fn description(&self) -> String {
        "Calling selfdestruct (or the deprecated alias suicide) permanently destroys the contract \
         and forwards its balance to an arbitrary address; remove this capability or guard it \
         behind strict multi-sig or governance controls to prevent accidental or malicious \
         destruction of contract state and funds."
            .to_string()
    }

    fn name(&self) -> String {
        "selfdestruct".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
