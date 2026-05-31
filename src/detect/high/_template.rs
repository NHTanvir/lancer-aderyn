//! `_template` — scaffolding for new high-severity detectors. NOT registered.
//!
//! Copy this file, rename the struct + `name()`, and add it to
//! `crate::detect::registry` to activate.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct TemplateHighDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for TemplateHighDetector {
    fn detect(&mut self, _context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Template High Detector".to_string()
    }

    fn description(&self) -> String {
        "Scaffolding detector — not active.".to_string()
    }

    fn name(&self) -> String {
        "template-high".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
