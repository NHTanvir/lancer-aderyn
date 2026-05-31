//! `unprotected-initializer` — flags public/external initializer functions with no modifiers.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::{is_constructor, is_public_or_external};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct UnprotectedInitializerDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for UnprotectedInitializerDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for f in context.function_definitions() {
            // Skip constructors — they are implicitly one-time-only.
            if is_constructor(f) {
                continue;
            }
            // Must be public or external to be callable by anyone.
            if !is_public_or_external(f) {
                continue;
            }
            // Name (lowercased) must start with "initialize" or equal "init".
            let lower = f.name.to_lowercase();
            if !lower.starts_with("initialize") && lower != "init" {
                continue;
            }
            // Flag only when the function carries NO modifiers at all —
            // i.e. there is no `initializer`, `onlyOwner`, or any other access guard.
            if f.modifiers.is_empty() {
                capture!(self, context, f);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Unprotected Initializer".to_string()
    }

    fn description(&self) -> String {
        "Initializer functions that are public or external and carry no access-control modifier \
         (such as `initializer`, `onlyOwner`, or a custom role guard) can be called by anyone at \
         any time; protect them with an appropriate modifier or access check so that \
         initialization can only happen once and only by an authorized caller."
            .to_string()
    }

    fn name(&self) -> String {
        "unprotected-initializer".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
