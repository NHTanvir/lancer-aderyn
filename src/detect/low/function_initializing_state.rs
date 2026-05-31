//! `function-initializing-state` — flags state variables initialized via a function call.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct FunctionInitializingStateDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for FunctionInitializingStateDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for v in context.variable_declarations() {
            // Only consider state variables that have an inline initializer.
            if !v.state_variable {
                continue;
            }
            if let Some(value) = &v.value {
                // The initializer must be a plain function call (not a type-conversion
                // or struct-constructor call).
                if let Expression::FunctionCall(call) = value {
                    let kind = call.kind.as_deref().unwrap_or("functionCall");
                    if kind == "functionCall" {
                        capture!(self, context, v);
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
        "State Initialized From a Function Call".to_string()
    }

    fn description(&self) -> String {
        "Initializing state variables with a function call can be error-prone because the function \
         may depend on state that is not yet set up at construction time; consider moving the \
         initialization into the constructor where execution order is explicit and well-defined."
            .to_string()
    }

    fn name(&self) -> String {
        "function-initializing-state".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
