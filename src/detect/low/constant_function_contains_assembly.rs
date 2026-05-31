//! `constant-function-contains-assembly` — flags view/pure functions whose bodies contain inline assembly.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::context::flow;
use crate::detect::helpers::is_view_or_pure;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct ConstantFunctionContainsAssemblyDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for ConstantFunctionContainsAssemblyDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for func in context.function_definitions() {
            if !is_view_or_pure(func) {
                continue;
            }
            let body = match &func.body {
                Some(b) => b,
                None => continue,
            };

            // Flatten all statements in the body (recursively) and look for any InlineAssembly.
            let mut stmts: Vec<Statement> = Vec::new();
            for stmt in &body.statements {
                flow::flatten_statements(stmt, &mut stmts);
            }

            let has_assembly = stmts
                .iter()
                .any(|s| matches!(s, Statement::InlineAssembly(_)));

            if has_assembly {
                capture!(self, context, func);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Constant Function Contains Assembly".to_string()
    }

    fn description(&self) -> String {
        "Inline assembly inside a `view` or `pure` function can read or write state in ways the \
         Solidity compiler cannot verify, silently violating the function's mutability guarantee; \
         refactor the logic to use pure Solidity, or if assembly is unavoidable, change the \
         function's mutability to `nonpayable` and document why."
            .to_string()
    }

    fn name(&self) -> String {
        "constant-function-contains-assembly".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
