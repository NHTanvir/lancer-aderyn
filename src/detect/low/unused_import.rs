//! `unused-import` — an import whose symbol never appears in the file.

use std::collections::{BTreeMap, HashSet};
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct UnusedImportDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for UnusedImportDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Names that appear anywhere as identifiers, members, base contracts, or types.
        let mut used: HashSet<String> = HashSet::new();
        for id in context.identifiers() {
            used.insert(id.name.clone());
        }
        for m in context.member_accesses() {
            used.insert(m.member_name.clone());
        }
        for c in context.contract_definitions() {
            for spec in &c.base_contracts {
                if let Some(b) = &spec.base_name {
                    used.insert(b.name.clone());
                }
            }
        }
        for v in context.variable_declarations() {
            if let Some(ts) = &v.type_descriptions.type_string {
                for tok in ts.split(|c: char| !c.is_alphanumeric() && c != '_') {
                    if !tok.is_empty() {
                        used.insert(tok.to_string());
                    }
                }
            }
        }

        for imp in context.import_directives() {
            let base = imp
                .file
                .rsplit('/')
                .next()
                .unwrap_or(&imp.file)
                .trim_end_matches(".sol");
            let candidate = if !imp.unit_alias.is_empty() {
                imp.unit_alias.clone()
            } else {
                base.to_string()
            };
            // Only consider symbol-like names (uppercase start) to limit false positives.
            let symbol_like = candidate
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false);
            if symbol_like && !used.contains(&candidate) {
                capture!(self, context, imp);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Unused Import".to_string()
    }

    fn description(&self) -> String {
        "This import does not appear to be used in the file. Remove it to reduce compilation \
         surface and improve readability."
            .to_string()
    }

    fn name(&self) -> String {
        "unused-import".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
