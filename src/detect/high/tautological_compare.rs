//! `tautological-compare` — flags comparisons where both sides are the same expression.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct TautologicalCompareDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Return true if the two expressions are "the same" by either:
///   1. Both are Identifiers referencing the same declaration (non-None), or
///   2. Their source text (via peek) is identical and non-empty.
fn expressions_are_same(
    ctx: &WorkspaceContext,
    left: &Expression,
    right: &Expression,
) -> bool {
    // Strategy 1: both Identifier nodes pointing at the same declaration.
    if let (Expression::Identifier(l_id), Expression::Identifier(r_id)) = (left, right) {
        if let (Some(l_decl), Some(r_decl)) = (l_id.referenced_declaration, r_id.referenced_declaration) {
            if l_decl == r_decl {
                return true;
            }
        }
    }

    // Strategy 2: source text peek match.
    let l_src = left.src();
    let r_src = right.src();
    if let (Some(l_text), Some(r_text)) = (ctx.peek(l_src), ctx.peek(r_src)) {
        if !l_text.is_empty() && l_text == r_text {
            return true;
        }
    }

    false
}

impl IssueDetector for TautologicalCompareDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for binop in context.binary_operations() {
            if !matches!(
                binop.operator.as_str(),
                "==" | "!=" | "<=" | ">=" | "<" | ">"
            ) {
                continue;
            }

            if expressions_are_same(
                context,
                &binop.left_expression,
                &binop.right_expression,
            ) {
                capture!(self, context, binop);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Tautological Comparison (x == x)".to_string()
    }

    fn description(&self) -> String {
        "A comparison where both operands are the same expression always evaluates to a constant \
         (true for `==`, `<=`, `>=`; false for `!=`, `<`, `>`), which indicates a logic error or \
         copy-paste mistake; remove the redundant check or replace it with the correct operand."
            .to_string()
    }

    fn name(&self) -> String {
        "tautological-compare".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
