//! `costly-operations-inside-a-loop` — flags loops that write contract storage on every iteration.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::context::browser::ApproximateStorageChangeFinder;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct CostlyLoopDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for CostlyLoopDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        let finder = ApproximateStorageChangeFinder::new(context);

        for stmt in context.for_statements() {
            if finder.statement_mutates_state(stmt.body.as_ref()) {
                capture!(self, context, stmt);
            }
        }

        for stmt in context.while_statements() {
            if finder.statement_mutates_state(stmt.body.as_ref()) {
                capture!(self, context, stmt);
            }
        }

        for stmt in context.do_while_statements() {
            if finder.statement_mutates_state(stmt.body.as_ref()) {
                capture!(self, context, stmt);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Costly Operation Inside a Loop".to_string()
    }

    fn description(&self) -> String {
        "Writing to contract storage inside a loop causes repeated SSTORE operations whose gas \
         cost scales with the number of iterations; refactor by accumulating results in a local \
         variable and writing to storage once after the loop completes."
            .to_string()
    }

    fn name(&self) -> String {
        "costly-operations-inside-a-loop".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
