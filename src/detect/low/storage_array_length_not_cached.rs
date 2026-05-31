//! `cache-array-length` — a loop condition reads a storage array's `.length`
//! on every iteration instead of caching it (repeated SLOAD).

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::flow;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct StorageArrayLengthNotCachedDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

fn base_is_state(context: &WorkspaceContext, expr: &Expression) -> bool {
    match expr {
        Expression::Identifier(id) => context.references_state_variable(id.referenced_declaration),
        Expression::MemberAccess(m) => base_is_state(context, &m.expression),
        Expression::IndexAccess(i) => base_is_state(context, &i.base_expression),
        _ => false,
    }
}

impl IssueDetector for StorageArrayLengthNotCachedDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for f in context.for_statements() {
            let Some(cond) = &f.condition else { continue };
            let mut exprs = Vec::new();
            flow::walk_expression(cond, &mut exprs);
            let reads_storage_length = exprs.iter().any(|e| {
                if let Expression::MemberAccess(m) = e {
                    m.member_name == "length" && base_is_state(context, &m.expression)
                } else {
                    false
                }
            });
            if reads_storage_length {
                capture!(self, context, f);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Storage Array Length Not Cached in Loop".to_string()
    }

    fn description(&self) -> String {
        "This loop reads a storage array's `.length` in its condition, performing an SLOAD on every \
         iteration. Cache the length in a local variable before the loop."
            .to_string()
    }

    fn name(&self) -> String {
        "cache-array-length".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
