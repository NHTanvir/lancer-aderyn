//! `internal-functions-called-once` — flags internal/private functions that are called exactly once.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::is_internal_or_private;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct InternalFunctionUsedOnceDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Build a reference-count map: for every `Identifier` and `MemberAccess`
/// whose `referenced_declaration` resolves to a `FunctionDefinition`, tally
/// how many times each function id is referenced.
fn build_ref_counts(context: &WorkspaceContext) -> BTreeMap<NodeID, usize> {
    let mut counts: BTreeMap<NodeID, usize> = BTreeMap::new();

    // Identifier references (e.g. direct bare calls or function-pointer usage).
    for ident in context.identifiers() {
        if let Some(ref_id) = ident.referenced_declaration {
            if context.resolve_function(ref_id).is_some() {
                *counts.entry(ref_id).or_insert(0) += 1;
            }
        }
    }

    // MemberAccess references (e.g. `LibraryName.funcName(...)` or
    // `this.funcName` — though internal calls rarely use member access,
    // this catches any edge cases where solc sets referenced_declaration).
    for ma in context.member_accesses() {
        if let Some(ref_id) = ma.referenced_declaration {
            if context.resolve_function(ref_id).is_some() {
                *counts.entry(ref_id).or_insert(0) += 1;
            }
        }
    }

    counts
}

impl IssueDetector for InternalFunctionUsedOnceDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        let ref_counts = build_ref_counts(context);

        for func in context.function_definitions() {
            // Only consider internal / private functions.
            if !is_internal_or_private(func) {
                continue;
            }
            // Skip constructors.
            if func.is_constructor() {
                continue;
            }
            // If this function is referenced exactly once it is a candidate for
            // being inlined at the call site, reducing code complexity.
            let count = ref_counts.get(&func.id).copied().unwrap_or(0);
            if count == 1 {
                capture!(self, context, func);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Internal Function Used Only Once".to_string()
    }

    fn description(&self) -> String {
        "Consider inlining internal or private functions that are only called from a single \
         location, as this removes the overhead of an extra JUMP and simplifies the code."
            .to_string()
    }

    fn name(&self) -> String {
        "internal-functions-called-once".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
