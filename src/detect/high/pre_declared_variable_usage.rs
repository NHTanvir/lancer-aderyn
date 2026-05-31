//! `pre-declared-local-variable-usage` — flags identifiers that reference a
//! local variable before its declaration site in the same function.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct PreDeclaredVariableUsageDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Return the 1-based line number for a node's `src` string, or `None` if the
/// source location cannot be resolved (unknown file, malformed src, etc.).
fn line_of(context: &WorkspaceContext, src: &str) -> Option<usize> {
    context.capture_key(src).map(|(_file, line, _snippet)| line)
}

impl IssueDetector for PreDeclaredVariableUsageDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // ── Step 1 ──────────────────────────────────────────────────────────
        // Build a map: declaration node-id  →  (declaration line, function id)
        // We only care about LOCAL variable declarations (state_variable == false).
        // Local declarations appear inside VariableDeclarationStatements.
        let mut local_decl_info: BTreeMap<NodeID, (usize, NodeID)> = BTreeMap::new();

        for vds in context.variable_declaration_statements() {
            for decl_opt in &vds.declarations {
                let decl = match decl_opt {
                    Some(d) => d,
                    None => continue,
                };
                // Only local variables (not state variables).
                if decl.state_variable {
                    continue;
                }
                // The declaration's line number.
                let decl_line = match line_of(context, &decl.src) {
                    Some(l) => l,
                    None => continue,
                };
                // The enclosing function (or modifier) id — used to confirm the
                // identifier is in the same function scope.
                let enclosing_fn_id = match context.enclosing_function(decl.id) {
                    Some(f) => f.id,
                    None => continue, // declaration outside a function — skip
                };
                local_decl_info.insert(decl.id, (decl_line, enclosing_fn_id));
            }
        }

        // ── Step 2 ──────────────────────────────────────────────────────────
        // Iterate every Identifier in the workspace.
        // If it references a local variable declared LATER (higher line number)
        // in the same function, that is a pre-declaration use.
        for ident in context.identifiers() {
            let ref_id = match ident.referenced_declaration {
                Some(id) => id,
                None => continue,
            };

            // Check if the referenced node is one of our tracked local decls.
            let (decl_line, decl_fn_id) = match local_decl_info.get(&ref_id) {
                Some(info) => *info,
                None => continue,
            };

            // The identifier must be inside the same function as the declaration.
            let ident_fn_id = match context.enclosing_function(ident.id) {
                Some(f) => f.id,
                None => continue,
            };
            if ident_fn_id != decl_fn_id {
                continue;
            }

            // Get the identifier's source line.
            let ident_line = match line_of(context, &ident.src) {
                Some(l) => l,
                None => continue,
            };

            // Flag only if the identifier appears textually before the declaration.
            if ident_line < decl_line {
                capture!(self, context, ident);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Variable Used Before Declaration".to_string()
    }

    fn description(&self) -> String {
        "A local variable is referenced before its declaration statement; move the declaration \
         to the top of the function (or before the first use) so the variable is always \
         initialised before it is read or written."
            .to_string()
    }

    fn name(&self) -> String {
        "pre-declared-local-variable-usage".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
