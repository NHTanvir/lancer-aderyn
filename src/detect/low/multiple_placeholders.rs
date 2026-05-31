//! `multiple-placeholders-in-modifier` — flags modifiers that contain more than one placeholder (`_`).

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::flow;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct MultiplePlaceholdersDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for MultiplePlaceholdersDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for m in context.modifier_definitions() {
            let body = match &m.body {
                Some(b) => b,
                None => continue,
            };

            // Flatten the entire modifier body into a linear list of statements.
            let wrapper = Statement::Block(body.clone());
            let mut all_stmts: Vec<Statement> = Vec::new();
            flow::flatten_statements(&wrapper, &mut all_stmts);

            // Count how many PlaceholderStatement occurrences exist.
            let placeholder_count = all_stmts
                .iter()
                .filter(|s| matches!(s, Statement::PlaceholderStatement(_)))
                .count();

            if placeholder_count > 1 {
                capture!(self, context, m);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Multiple Placeholders in Modifier".to_string()
    }

    fn description(&self) -> String {
        "A modifier should contain exactly one placeholder (`_`); using multiple placeholders \
         causes the modified function body to execute more than once, which can lead to \
         unexpected re-entrancy, duplicate state changes, or incorrect access-control behaviour \
         — refactor the modifier to invoke `_` only once."
            .to_string()
    }

    fn name(&self) -> String {
        "multiple-placeholders-in-modifier".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
