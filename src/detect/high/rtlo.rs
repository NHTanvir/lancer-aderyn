//! `rtlo` — Right-to-Left Override character in source code.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::SrcLocation;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

use crate::ast::NodeID;

#[derive(Default)]
pub struct RtloDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for RtloDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for file in &context.files {
            for (i, line) in file.content.lines().enumerate() {
                if line.contains('\u{202E}') || line.contains('\u{202D}') {
                    let key: CaptureKey = (file.path.clone(), i + 1, line.trim().to_string());
                    // Find the SourceUnit whose source_index matches this file's source_index.
                    let node_id = context
                        .source_units()
                        .iter()
                        .find(|u| {
                            SrcLocation::parse(&u.src)
                                .map(|loc| loc.source_index == file.source_index)
                                .unwrap_or(false)
                        })
                        .map(|u| u.id)
                        .unwrap_or(0);
                    self.found_instances.insert(key, node_id);
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Right-to-Left Override Character".to_string()
    }

    fn description(&self) -> String {
        "Remove all Right-to-Left Override (U+202E) and Left-to-Right Override (U+202D) \
         characters from source files; these invisible Unicode characters can be used to \
         disguise malicious code by reversing the visual display of surrounding text."
            .to_string()
    }

    fn name(&self) -> String {
        "rtlo".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
