//! `missing-inheritance` — contract implements all functions of an interface but does not declare it.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::{functions_of_contract, is_interface, is_public_or_external};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct MissingInheritanceDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for MissingInheritanceDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Step 1: collect interface id -> set of public/external function names.
        // Only keep interfaces with >= 2 qualifying functions (reduces noise).
        let mut interface_functions: HashMap<NodeID, BTreeSet<String>> = HashMap::new();

        for contract in context.contract_definitions() {
            if !is_interface(contract) {
                continue;
            }
            let pub_ext_names: BTreeSet<String> = functions_of_contract(contract)
                .into_iter()
                .filter(|f| is_public_or_external(f) && f.kind_str() == "function")
                .map(|f| f.name.clone())
                .collect();

            if pub_ext_names.len() >= 2 {
                interface_functions.insert(contract.id, pub_ext_names);
            }
        }

        // Step 2: for each non-interface contract, collect its public/external function names.
        // Then check whether it fully covers any interface that is NOT already in its
        // linearized_base_contracts.
        for contract in context.contract_definitions() {
            if is_interface(contract) {
                continue;
            }

            // Collect all public/external function names on this contract.
            let contract_fn_names: BTreeSet<String> = functions_of_contract(contract)
                .into_iter()
                .filter(|f| is_public_or_external(f) && f.kind_str() == "function")
                .map(|f| f.name.clone())
                .collect();

            // The set of base contract IDs already declared.
            let already_inherits: BTreeSet<NodeID> =
                contract.linearized_base_contracts.iter().copied().collect();

            for (iface_id, iface_fns) in &interface_functions {
                // Skip if already inherited (including transitively).
                if already_inherits.contains(iface_id) {
                    continue;
                }

                // Capture if every interface function name is present in the contract.
                if iface_fns.iter().all(|name| contract_fn_names.contains(name)) {
                    capture!(self, context, contract);
                    // One hit per contract is enough; break to avoid duplicates.
                    break;
                }
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Missing Interface Inheritance".to_string()
    }

    fn description(&self) -> String {
        "The contract implements all functions required by an interface but does not explicitly \
         inherit from it; add the interface to the contract's inheritance list (e.g. \
         `contract Foo is IFoo`) so that the compiler enforces the full ABI and tooling can \
         recognise the conformance."
            .to_string()
    }

    fn name(&self) -> String {
        "missing-inheritance".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
