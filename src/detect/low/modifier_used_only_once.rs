//! `modifier-used-only-once` — flags modifiers that are applied to exactly one function.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct ModifierUsedOnlyOnceDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for ModifierUsedOnlyOnceDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Build a usage-count map: modifier name -> number of functions that apply it.
        let mut usage_count: BTreeMap<String, usize> = BTreeMap::new();

        for func in context.function_definitions() {
            for invocation in &func.modifiers {
                if let Some(path) = &invocation.modifier_name {
                    *usage_count.entry(path.name.clone()).or_insert(0) += 1;
                }
            }
        }

        // Capture every modifier definition whose name appears exactly once.
        for modifier in context.modifier_definitions() {
            let count = usage_count.get(&modifier.name).copied().unwrap_or(0);
            if count == 1 {
                capture!(self, context, modifier);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Modifier Used Only Once".to_string()
    }

    fn description(&self) -> String {
        "Consider inlining the modifier's logic directly into the single function that uses it, \
         or verify whether the modifier was intended to guard additional functions."
            .to_string()
    }

    fn name(&self) -> String {
        "modifier-used-only-once".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
