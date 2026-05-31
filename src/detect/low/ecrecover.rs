//! `ecrecover` — flags direct use of the built-in ecrecover function.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct EcrecoverDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for EcrecoverDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for call in context.function_calls() {
            if let Expression::Identifier(id) = call.expression.as_ref() {
                if id.name == "ecrecover" {
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
        "Direct Use of ecrecover".to_string()
    }

    fn description(&self) -> String {
        "Using ecrecover directly is error-prone because it returns address(0) on failure and is \
         vulnerable to signature malleability; replace it with OpenZeppelin's ECDSA library, which \
         reverts on failure and normalises the signature to prevent malleability attacks."
            .to_string()
    }

    fn name(&self) -> String {
        "ecrecover".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
