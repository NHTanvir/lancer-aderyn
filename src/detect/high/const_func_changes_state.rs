//! `const-func-changes-state` — view/pure function that mutates contract storage or uses inline assembly.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::browser::ApproximateStorageChangeFinder;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::context::flow;
use crate::detect::helpers::is_view_or_pure;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct ConstFuncChangesStateDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if any statement in the flattened list of `body` statements is
/// an `InlineAssembly` node.
fn body_contains_inline_assembly(body: &Block) -> bool {
    let wrapper = Statement::Block(body.clone());
    let mut all_stmts = Vec::new();
    flow::flatten_statements(&wrapper, &mut all_stmts);
    all_stmts
        .iter()
        .any(|s| matches!(s, Statement::InlineAssembly(_)))
}

impl IssueDetector for ConstFuncChangesStateDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        let finder = ApproximateStorageChangeFinder::new(context);

        for func in context.function_definitions() {
            // Only check view/pure functions that have a body.
            if !is_view_or_pure(func) {
                continue;
            }
            if let Some(body) = &func.body {
                if finder.block_mutates_state(body) || body_contains_inline_assembly(body) {
                    capture!(self, context, func);
                }
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "View/Pure Function Changes State".to_string()
    }

    fn description(&self) -> String {
        "Functions declared `view` or `pure` must not modify contract state or use inline \
         assembly; remove the `view`/`pure` modifier or refactor the function so that it does \
         not write to storage."
            .to_string()
    }

    fn name(&self) -> String {
        "const-func-changes-state".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
