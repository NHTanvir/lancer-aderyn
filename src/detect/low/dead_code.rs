//! `dead-code` — flags internal/private functions that are never referenced.

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct DeadCodeDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for DeadCodeDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // --- Step 1: collect every function NodeID that is referenced somewhere ---

        let mut referenced: HashSet<NodeID> = HashSet::new();

        // Identifiers that point directly at a FunctionDefinition (internal calls,
        // function-value references, using-for dispatch, etc.).
        for ident in context.identifiers() {
            if let Some(decl_id) = ident.referenced_declaration {
                if let Some(AstNode::FunctionDefinition(_)) = context.get_node(decl_id) {
                    referenced.insert(decl_id);
                }
            }
        }

        // MemberAccess nodes can also resolve to a FunctionDefinition
        // (e.g. `LibName.helper(...)` or `super.foo()`).
        for ma in context.member_accesses() {
            if let Some(decl_id) = ma.referenced_declaration {
                if let Some(AstNode::FunctionDefinition(_)) = context.get_node(decl_id) {
                    referenced.insert(decl_id);
                }
            }
        }

        // base_functions: if function A overrides B, A lists B in base_functions.
        // Those base functions are therefore "used" (they define the interface).
        for func in context.function_definitions() {
            for &base_id in &func.base_functions {
                referenced.insert(base_id);
            }
        }

        // --- Step 2: flag internal/private functions not in the referenced set ---

        for func in context.function_definitions() {
            // Only flag internal or private functions.
            let vis = func.visibility_str();
            if vis != "internal" && vis != "private" {
                continue;
            }
            // Constructors are always "used" (called by the EVM on deployment).
            if func.is_constructor() {
                continue;
            }
            // Virtual functions act as base declarations for override chains;
            // they may legitimately have no direct call sites.
            if func.is_virtual {
                continue;
            }
            // Functions that themselves override something are referenced via
            // their base_functions entry, already handled above.
            if func.overrides.is_some() {
                continue;
            }
            // If the function id does not appear in the referenced set, report it.
            if !referenced.contains(&func.id) {
                capture!(self, context, func);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Dead Code (Unused Internal Function)".to_string()
    }

    fn description(&self) -> String {
        "This internal or private function is never called within the contract or any derived \
         contract; remove it to reduce deployment cost and improve code clarity."
            .to_string()
    }

    fn name(&self) -> String {
        "dead-code".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
