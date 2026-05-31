//! `function-selector-collision` — detects two or more public/external functions sharing the same 4-byte selector.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::is_public_or_external;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct FunctionSelectorCollisionDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for FunctionSelectorCollisionDetector {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Map each 4-byte selector hex string -> list of (function_name, &FunctionDefinition)
        // for every public/external function that has a selector assigned by solc.
        let mut selector_map: BTreeMap<String, Vec<(String, &FunctionDefinition)>> =
            BTreeMap::new();

        for func in ctx.function_definitions() {
            // Only public/external functions participate in ABI dispatch.
            if !is_public_or_external(func) {
                continue;
            }
            // Constructors, fallback, and receive have no selector.
            if func.is_constructor() {
                continue;
            }
            if matches!(func.kind_str(), "fallback" | "receive") {
                continue;
            }
            if let Some(sel) = &func.function_selector {
                if sel.is_empty() {
                    continue;
                }
                selector_map
                    .entry(sel.clone())
                    .or_default()
                    .push((func.name.clone(), func));
            }
        }

        // Any selector mapped to 2+ DISTINCT function names is a collision.
        for (_selector, entries) in &selector_map {
            // Collect the distinct names in this group.
            let mut seen_names: Vec<&str> = Vec::new();
            for (name, _) in entries.iter() {
                if !seen_names.contains(&name.as_str()) {
                    seen_names.push(name.as_str());
                }
            }
            if seen_names.len() >= 2 {
                // Capture every function in this collision group.
                for (_, func) in entries.iter() {
                    capture!(self, ctx, *func);
                }
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Function Selector Collision".to_string()
    }

    fn description(&self) -> String {
        "Two or more public/external functions share the same 4-byte ABI selector; the EVM \
         dispatcher will route all matching calls to the first match, silently ignoring the \
         others — rename one of the colliding functions to give it a unique selector."
            .to_string()
    }

    fn name(&self) -> String {
        "function-selector-collision".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
