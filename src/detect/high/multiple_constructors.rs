//! `multiple-constructors` — flags contracts that define more than one constructor-like function.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::functions_of_contract;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct MultipleConstructorsDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for MultipleConstructorsDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for contract in context.contract_definitions() {
            let constructor_count = functions_of_contract(contract)
                .iter()
                .filter(|f| f.is_constructor() || f.name == contract.name)
                .count();

            if constructor_count >= 2 {
                capture!(self, context, contract);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Multiple Constructors".to_string()
    }

    fn description(&self) -> String {
        "A contract must have exactly one constructor; remove all duplicate constructor-like \
         functions (old-style constructors sharing the contract name) and consolidate \
         initialization logic into a single `constructor()` definition."
            .to_string()
    }

    fn name(&self) -> String {
        "multiple-constructors".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
