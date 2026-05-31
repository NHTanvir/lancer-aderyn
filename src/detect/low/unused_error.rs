//! `unused-error` — a custom error that is declared but never used.

use std::collections::{BTreeMap, HashSet};
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct UnusedErrorDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for UnusedErrorDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        let mut referenced: HashSet<NodeID> = HashSet::new();
        for id in context.identifiers() {
            if let Some(r) = id.referenced_declaration {
                referenced.insert(r);
            }
        }
        for m in context.member_accesses() {
            if let Some(r) = m.referenced_declaration {
                referenced.insert(r);
            }
        }

        for e in context.error_definitions() {
            if !referenced.contains(&e.id) {
                capture!(self, context, e);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Unused Custom Error".to_string()
    }

    fn description(&self) -> String {
        "This custom error is declared but never reverted with. Remove it, or wire it into the \
         relevant revert path."
            .to_string()
    }

    fn name(&self) -> String {
        "unused-error".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
