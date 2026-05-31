//! `state-change-without-event` — public/external state-mutating functions that emit no event.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::browser::ApproximateStorageChangeFinder;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::{is_public_or_external, is_view_or_pure};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct StateChangeWithoutEventDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Recursively collect all statements from a `Block` into `out`.
fn flatten_block(block: &Block, out: &mut Vec<Statement>) {
    for stmt in &block.statements {
        flatten_statement(stmt, out);
    }
}

/// Recursively collect `stmt` and all nested statements into `out`.
fn flatten_statement(stmt: &Statement, out: &mut Vec<Statement>) {
    out.push(stmt.clone());
    match stmt {
        Statement::Block(b) => {
            for s in &b.statements {
                flatten_statement(s, out);
            }
        }
        Statement::UncheckedBlock(u) => {
            for s in &u.statements {
                flatten_statement(s, out);
            }
        }
        Statement::IfStatement(i) => {
            flatten_statement(&i.true_body, out);
            if let Some(fb) = &i.false_body {
                flatten_statement(fb, out);
            }
        }
        Statement::ForStatement(f) => {
            flatten_statement(&f.body, out);
        }
        Statement::WhileStatement(w) => {
            flatten_statement(&w.body, out);
        }
        Statement::DoWhileStatement(d) => {
            flatten_statement(&d.body, out);
        }
        Statement::TryStatement(t) => {
            for clause in &t.clauses {
                flatten_block(&clause.block, out);
            }
        }
        _ => {}
    }
}

/// Returns true if any statement in the flattened list is an `EmitStatement`.
fn block_has_emit(block: &Block) -> bool {
    let mut stmts = Vec::new();
    flatten_block(block, &mut stmts);
    stmts
        .iter()
        .any(|s| matches!(s, Statement::EmitStatement(_)))
}

impl IssueDetector for StateChangeWithoutEventDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        let finder = ApproximateStorageChangeFinder::new(context);

        for func in context.function_definitions() {
            // Only public/external functions matter — internal callers are not entry points.
            if !is_public_or_external(func) {
                continue;
            }
            // view/pure cannot write state; skip them.
            if is_view_or_pure(func) {
                continue;
            }
            // Must have an implemented body.
            let body = match &func.body {
                Some(b) => b,
                None => continue,
            };

            // Only flag if the body actually mutates state storage.
            if !finder.block_mutates_state(body) {
                continue;
            }

            // If there is at least one emit anywhere in the body, it is fine.
            if block_has_emit(body) {
                continue;
            }

            capture!(self, context, func);
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "State Change Without Event".to_string()
    }

    fn description(&self) -> String {
        "Emit an event for every public or external function that modifies contract state so that \
         off-chain consumers and monitoring tools can reliably track state transitions without \
         replaying transactions."
            .to_string()
    }

    fn name(&self) -> String {
        "state-change-without-event".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
