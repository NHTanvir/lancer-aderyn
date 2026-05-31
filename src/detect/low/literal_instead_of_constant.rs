//! `literal-instead-of-constant` — flags magic number literals in binary operations.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct LiteralInsteadOfConstantDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if the literal is a hex number (starts with "0x" or "0X").
fn is_hex_literal(lit: &Literal) -> bool {
    if let Some(val) = &lit.value {
        if val.starts_with("0x") || val.starts_with("0X") {
            return true;
        }
    }
    // If hex_value is present but value is absent it is a hex literal.
    if lit.value.is_none() && lit.hex_value.is_some() {
        return true;
    }
    false
}

/// Returns true if the literal is a trivially common number (0, 1, 2).
fn is_trivial_value(val: &str) -> bool {
    matches!(val, "0" | "1" | "2")
}

/// Returns true if the enclosing VariableDeclaration (if any) is constant or immutable.
fn inside_constant_or_immutable_var_decl(ctx: &WorkspaceContext, node_id: NodeID) -> bool {
    if let Some(ancestor) = ctx.closest_ancestor_of_type(node_id, "VariableDeclaration") {
        if let AstNode::VariableDeclaration(vd) = ancestor {
            return vd.is_constant() || vd.is_immutable();
        }
    }
    false
}

/// Check one operand expression: if it is a number Literal that warrants flagging,
/// return a cloned `Expression::Literal(...)` wrapping it.
fn check_operand(ctx: &WorkspaceContext, expr: &Expression) -> Option<Expression> {
    if let Expression::Literal(lit) = expr {
        if lit.kind.as_deref() != Some("number") {
            return None;
        }
        if is_hex_literal(lit) {
            return None;
        }
        let val = lit.value.as_deref()?;
        if is_trivial_value(val) {
            return None;
        }
        // Only flag multi-digit literals (length >= 2, i.e., value >= 10).
        if val.len() < 2 {
            return None;
        }
        // Skip if inside a constant / immutable variable declaration.
        if inside_constant_or_immutable_var_decl(ctx, lit.id) {
            return None;
        }
        return Some(Expression::Literal(lit.clone()));
    }
    None
}

impl IssueDetector for LiteralInsteadOfConstantDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for binop in context.binary_operations() {
            // Check left operand.
            if let Some(lit_expr) = check_operand(context, binop.left_expression.as_ref()) {
                capture!(self, context, &lit_expr);
            }
            // Check right operand.
            if let Some(lit_expr) = check_operand(context, binop.right_expression.as_ref()) {
                capture!(self, context, &lit_expr);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Magic Number (Use a Constant)".to_string()
    }

    fn description(&self) -> String {
        "Replace magic number literals in expressions with a named `constant` or `immutable` \
         variable so that the intent is self-documenting and the value can be updated in a single \
         place without risk of inconsistency."
            .to_string()
    }

    fn name(&self) -> String {
        "literal-instead-of-constant".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
