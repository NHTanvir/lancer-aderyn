//! `state-variable-read-external` — reading a state variable through an external
//! `this.<getter>()` call instead of directly (wastes gas / does a CALL).

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct StateVariableReadExternalDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for StateVariableReadExternalDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for call in context.function_calls() {
            if let Expression::MemberAccess(ma) = call.expression.as_ref() {
                if let Expression::Identifier(base) = ma.expression.as_ref() {
                    if base.name == "this" {
                        capture!(self, context, call);
                    }
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "State Variable Read via External Call".to_string()
    }

    fn description(&self) -> String {
        "Reading a value through `this.<member>()` performs an external CALL and re-loads from \
         storage; read the state variable or call the internal function directly instead."
            .to_string()
    }

    fn name(&self) -> String {
        "state-variable-read-external".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
