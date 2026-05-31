//! `storage-array-edit-with-memory` — flags local memory copies of storage arrays/structs
//! that silently discard any mutations made to the copy.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct StorageArrayMemoryEditDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Return true if the expression's initial value references a state variable.
/// Handles: plain Identifier, MemberAccess on a state var, and IndexAccess
/// whose base ultimately resolves to a state variable.
fn initial_value_references_state(expr: &Expression, ctx: &WorkspaceContext) -> bool {
    match expr {
        Expression::Identifier(id) => ctx.references_state_variable(id.referenced_declaration),
        Expression::MemberAccess(ma) => {
            // e.g. `this.myArray` or `someStateStruct.field`
            if ctx.references_state_variable(ma.referenced_declaration) {
                return true;
            }
            initial_value_references_state(&ma.expression, ctx)
        }
        Expression::IndexAccess(ia) => {
            // e.g. `myStorageMapping[key]` — base must resolve to state
            initial_value_references_state(&ia.base_expression, ctx)
        }
        Expression::IndexRangeAccess(ira) => {
            initial_value_references_state(&ira.base_expression, ctx)
        }
        _ => false,
    }
}

/// True if the type string indicates an array or a struct (both are reference
/// types whose memory copy will NOT write back to storage).
fn type_is_array_or_struct(type_str: &str) -> bool {
    type_str.contains("[]") || type_str.contains('[') || type_str.contains("struct ")
}

impl IssueDetector for StorageArrayMemoryEditDetector {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for stmt in ctx.variable_declaration_statements() {
            // Only care about the initial value — if there's none, skip.
            let init = match &stmt.initial_value {
                Some(iv) => iv,
                None => continue,
            };

            // Walk every declared variable in this statement.
            for decl in stmt.declarations.iter().flatten() {
                // Must be a `memory` local (not storage, not calldata).
                if decl.storage_location.as_deref() != Some("memory") {
                    continue;
                }

                // The declared type must be an array or struct.
                let is_ref_type = decl
                    .type_descriptions
                    .type_string
                    .as_deref()
                    .map(type_is_array_or_struct)
                    .unwrap_or(false)
                    || decl
                        .type_name
                        .as_ref()
                        .and_then(|tn| tn.type_string())
                        .map(type_is_array_or_struct)
                        .unwrap_or(false);

                if !is_ref_type {
                    continue;
                }

                // The RHS must come from storage.
                if initial_value_references_state(init, ctx) {
                    capture!(self, ctx, stmt);
                    // Only capture the statement once even if multiple decls match.
                    break;
                }
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Storage Array Edited via Memory Copy".to_string()
    }

    fn description(&self) -> String {
        "Assigning a storage array or struct to a `memory` variable creates an in-memory copy; \
         any modifications to that copy are never written back to storage. \
         Use a `storage` pointer (e.g. `MyStruct storage ref = myStorageVar;`) \
         to mutate the original storage slot directly."
            .to_string()
    }

    fn name(&self) -> String {
        "storage-array-edit-with-memory".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
