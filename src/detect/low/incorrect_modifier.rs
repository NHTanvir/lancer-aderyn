//! `incorrect-modifier` — flags modifier bodies that contain no placeholder (`_`).

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::context::flow;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct IncorrectModifierDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for IncorrectModifierDetector {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for m in ctx.modifier_definitions() {
            let body = match &m.body {
                Some(b) => b,
                None => continue,
            };

            // Flatten the entire modifier body into a linear list of statements.
            let wrapper = Statement::Block(body.clone());
            let mut all_stmts: Vec<Statement> = Vec::new();
            flow::flatten_statements(&wrapper, &mut all_stmts);

            // If there is no PlaceholderStatement anywhere, the modifier never
            // transfers control to the guarded function body — flag it.
            let has_placeholder = all_stmts
                .iter()
                .any(|s| matches!(s, Statement::PlaceholderStatement(_)));

            if !has_placeholder {
                capture!(self, ctx, m);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Modifier Missing Placeholder".to_string()
    }

    fn description(&self) -> String {
        "Every modifier body should contain a placeholder statement (`_`) to transfer control to \
         the modified function; a modifier without `_` will silently skip the function body, \
         causing unexpected behavior — add `_` at the correct position in the modifier."
            .to_string()
    }

    fn name(&self) -> String {
        "incorrect-modifier".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
