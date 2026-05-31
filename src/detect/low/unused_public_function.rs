//! `unused-public-function` — a `public` function never referenced internally
//! (and not an interface/override entrypoint).

use std::collections::{BTreeMap, HashSet};
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::{is_interface, is_library};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct UnusedPublicFunctionDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for UnusedPublicFunctionDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        let mut referenced: HashSet<NodeID> = HashSet::new();
        for id in context.identifiers() {
            if let Some(r) = id.referenced_declaration {
                referenced.insert(r);
            }
        }
        for m in context.member_accesses() {
            if let Some(r) = m.referenced_declaration {
                referenced.insert(r);
            }
        }
        for f in context.function_definitions() {
            for b in &f.base_functions {
                referenced.insert(*b);
            }
        }

        for f in context.function_definitions() {
            if f.visibility_str() != "public" || f.kind_str() != "function" {
                continue;
            }
            if f.is_virtual || f.overrides.is_some() || !f.base_functions.is_empty() {
                continue;
            }
            if f.body.is_none() {
                continue;
            }
            if let Some(c) = context.enclosing_contract(f.id) {
                if is_interface(c) || is_library(c) || c.is_abstract {
                    continue;
                }
            }
            if !referenced.contains(&f.id) {
                capture!(self, context, f);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Unused Public Function".to_string()
    }

    fn description(&self) -> String {
        "This `public` function is never called internally. If it is not part of the external API, \
         remove it; if it is only called externally, mark it `external` to save gas."
            .to_string()
    }

    fn name(&self) -> String {
        "unused-public-function".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
