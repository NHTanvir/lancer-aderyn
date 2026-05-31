//! `uninitialized-local-variable` — a local variable declared without an
//! initial value.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct UninitializedLocalVariableDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for UninitializedLocalVariableDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for stmt in context.variable_declaration_statements() {
            if stmt.initial_value.is_some() {
                continue;
            }
            let has_local = stmt
                .declarations
                .iter()
                .flatten()
                .any(|d| !d.state_variable);
            if has_local {
                capture!(self, context, stmt);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Uninitialized Local Variable".to_string()
    }

    fn description(&self) -> String {
        "This local variable is declared without an initial value and defaults to zero. Initialize \
         it explicitly to avoid relying on the implicit default."
            .to_string()
    }

    fn name(&self) -> String {
        "uninitialized-local-variable".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
