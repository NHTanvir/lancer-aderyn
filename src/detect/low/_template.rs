//! `_template` — scaffolding for new low-severity detectors. NOT registered.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct TemplateLowDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for TemplateLowDetector {
    fn detect(&mut self, _context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Template Low Detector".to_string()
    }

    fn description(&self) -> String {
        "Scaffolding detector — not active.".to_string()
    }

    fn name(&self) -> String {
        "template-low".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
