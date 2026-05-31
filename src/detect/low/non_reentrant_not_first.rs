//! `non-reentrant-not-first` — flags functions where `nonReentrant` is not the first modifier.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct NonReentrantNotFirstDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for NonReentrantNotFirstDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for f in context.function_definitions() {
            // Find the index of any modifier whose name (lowercased) contains "nonreentrant".
            let position = f.modifiers.iter().position(|m| {
                m.modifier_name
                    .as_ref()
                    .map(|id| id.name.to_lowercase().contains("nonreentrant"))
                    .unwrap_or(false)
            });

            // Flag if found and it is NOT the first modifier (index != 0).
            if let Some(idx) = position {
                if idx != 0 {
                    capture!(self, context, f);
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "nonReentrant Not First Modifier".to_string()
    }

    fn description(&self) -> String {
        "Place the `nonReentrant` modifier as the first modifier on a function so that the \
         reentrancy guard is checked before any other modifier runs, preventing potential \
         reentrancy through modifier side-effects."
            .to_string()
    }

    fn name(&self) -> String {
        "non-reentrant-not-first".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
