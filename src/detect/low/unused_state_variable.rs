//! `unused-state-variable` — a non-public state variable never read or written.

use std::collections::{BTreeMap, HashSet};
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct UnusedStateVariableDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for UnusedStateVariableDetector {
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

        for v in context.variable_declarations() {
            if !v.state_variable {
                continue;
            }
            // Public state variables expose an automatic getter, so they count as used.
            if v.visibility.as_deref() == Some("public") {
                continue;
            }
            if !referenced.contains(&v.id) {
                capture!(self, context, v);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Unused State Variable".to_string()
    }

    fn description(&self) -> String {
        "This state variable is never read or written. Remove it to free its storage slot and \
         reduce contract size."
            .to_string()
    }

    fn name(&self) -> String {
        "unused-state-variable".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
