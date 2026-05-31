//! `todo` — flags `TODO` / `FIXME` markers left in the source.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct TodoDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for TodoDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for file in &context.files {
            // Best-effort: the source unit id covering this file (for tooling).
            let unit_id = context
                .source_units()
                .iter()
                .find(|u| {
                    SrcLocation::parse(&u.src)
                        .map(|l| l.source_index == file.source_index)
                        .unwrap_or(false)
                })
                .map(|u| u.id)
                .unwrap_or(0);

            for (i, line) in file.content.lines().enumerate() {
                let has_marker = line.contains("TODO") || line.contains("FIXME");
                let in_comment = line.contains("//") || line.contains("/*") || line.contains('*');
                if has_marker && in_comment {
                    let key = (file.path.clone(), i + 1, line.trim().to_string());
                    self.found_instances.insert(key, unit_id);
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "TODO Left in Code".to_string()
    }

    fn description(&self) -> String {
        "A TODO/FIXME marker remains in the source. Resolve it (or track it elsewhere) before \
         shipping — unfinished logic is a common source of audit findings."
            .to_string()
    }

    fn name(&self) -> String {
        "todo".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
