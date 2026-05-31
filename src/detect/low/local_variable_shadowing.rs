//! `local-variable-shadowing` — flags local variables whose names shadow a state variable.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::state_variables_of_contract;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct LocalVariableShadowingDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for LocalVariableShadowingDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for v in context.variable_declarations() {
            // Only consider local variables (not state variables).
            if v.state_variable {
                continue;
            }
            // Skip unnamed variables (e.g. blank tuple slots).
            if v.name.is_empty() {
                continue;
            }
            // Find the enclosing contract; skip free functions / file-level vars.
            let contract = match context.enclosing_contract(v.id) {
                Some(c) => c,
                None => continue,
            };
            // Collect all state variable names declared in the contract.
            let state_names: Vec<&str> = state_variables_of_contract(contract)
                .into_iter()
                .map(|sv| sv.name.as_str())
                .collect();
            // If the local name matches any state variable name, flag it.
            if state_names.contains(&v.name.as_str()) {
                capture!(self, context, v);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Local Variable Shadows State Variable".to_string()
    }

    fn description(&self) -> String {
        "Rename the local variable so it does not share its name with a contract state variable; \
         shadowing makes the code harder to reason about and can hide unintended reads or writes \
         to the wrong variable."
            .to_string()
    }

    fn name(&self) -> String {
        "local-variable-shadowing".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
