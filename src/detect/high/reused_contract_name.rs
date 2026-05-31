//! `reused-contract-name` — flags contract names that appear more than once across the codebase.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct ReusedContractNameDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for ReusedContractNameDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Group all contract definitions by their name.
        let mut by_name: BTreeMap<String, Vec<&ContractDefinition>> = BTreeMap::new();
        for c in context.contract_definitions() {
            by_name.entry(c.name.clone()).or_default().push(c);
        }

        // For every name that appears in two or more contracts, capture each occurrence.
        for (_name, contracts) in &by_name {
            if contracts.len() >= 2 {
                for c in contracts {
                    capture!(self, context, *c);
                }
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Reused Contract Name".to_string()
    }

    fn description(&self) -> String {
        "Multiple contracts share the same name, which causes ambiguity in tooling, audits, and \
         on-chain verification; rename each contract to a unique, descriptive identifier."
            .to_string()
    }

    fn name(&self) -> String {
        "reused-contract-name".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
