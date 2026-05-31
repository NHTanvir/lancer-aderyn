//! `state-variable-shadowing` — detects child contract state variables that shadow a base contract's state variable.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::state_variables_of_contract;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct StateVariableShadowingDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for StateVariableShadowingDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for child_contract in context.contract_definitions() {
            // Collect the names of state variables declared directly on the child.
            let child_vars = state_variables_of_contract(child_contract);

            // linearized_base_contracts[0] is the contract itself; skip it.
            let base_ids = child_contract
                .linearized_base_contracts
                .iter()
                .skip(1)
                .copied()
                .collect::<Vec<_>>();

            if base_ids.is_empty() {
                continue;
            }

            // Collect all state-variable names declared in any base contract.
            let mut base_var_names: std::collections::HashSet<String> =
                std::collections::HashSet::new();

            for base_id in &base_ids {
                if let Some(base_contract) = context
                    .contract_definitions()
                    .iter()
                    .find(|c| c.id == *base_id)
                {
                    for base_var in state_variables_of_contract(base_contract) {
                        base_var_names.insert(base_var.name.clone());
                    }
                }
            }

            // Flag any child state variable whose name clashes with a base one.
            for child_var in child_vars {
                if base_var_names.contains(&child_var.name) {
                    capture!(self, context, child_var);
                }
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "State Variable Shadowing".to_string()
    }

    fn description(&self) -> String {
        "A state variable in a derived contract shares its name with a state variable declared in \
         one of its base contracts; rename the variable in the derived contract (or the base) so \
         that each storage slot has a unique, unambiguous identifier and accesses cannot silently \
         target the wrong slot."
            .to_string()
    }

    fn name(&self) -> String {
        "state-variable-shadowing".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
