//! `empty-block` — flags functions / blocks with empty bodies.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct EmptyBlockDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for EmptyBlockDetector {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for func in ctx.function_definitions() {
            // Constructors and receive/fallback are commonly intentionally empty.
            if func.is_constructor() {
                continue;
            }
            if let Some(body) = &func.body {
                if body.statements.is_empty() {
                    // Skip empty bodies that only forward to a base via modifiers.
                    if func.modifiers.is_empty() && func.kind_str() == "function" {
                        capture!(self, ctx, func);
                    }
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Empty Block".to_string()
    }

    fn description(&self) -> String {
        "Consider removing empty blocks or emitting an event / adding a comment explaining why the \
         block is intentionally empty."
            .to_string()
    }

    fn name(&self) -> String {
        "empty-block".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
