//! `delete-nested-mapping` — flags `delete` applied to a struct reached through a mapping index.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct DeleteNestedMappingDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true when `expr` is an `IndexAccess` whose base expression's type
/// string contains "mapping(" — i.e. the delete target is reached by indexing
/// into a mapping.
fn target_is_mapping_index(expr: &Expression) -> bool {
    if let Expression::IndexAccess(ia) = expr {
        if let Some(ts) = ia.base_expression.type_string() {
            return ts.contains("mapping(");
        }
    }
    false
}

impl IssueDetector for DeleteNestedMappingDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for u in context.unary_operations() {
            // Only care about the `delete` keyword.
            if u.operator != "delete" {
                continue;
            }
            // The deleted expression's type must contain "struct" (it is a
            // struct value stored inside the mapping).
            let sub_type = u.sub_expression.type_string().unwrap_or("");
            if !sub_type.contains("struct") {
                continue;
            }
            // The sub-expression must be an IndexAccess whose base is a mapping.
            if target_is_mapping_index(&u.sub_expression) {
                capture!(self, context, u);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Deletion of Nested Mapping".to_string()
    }

    fn description(&self) -> String {
        "Using `delete` on a struct value obtained from a mapping only resets the struct's \
         non-mapping fields to their defaults; any nested mappings inside the struct are \
         unaffected and retain their data, leading to a corrupted or leaking storage state. \
         Instead of relying on `delete`, manually reset every nested mapping entry before \
         removing the parent struct."
            .to_string()
    }

    fn name(&self) -> String {
        "delete-nested-mapping".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
