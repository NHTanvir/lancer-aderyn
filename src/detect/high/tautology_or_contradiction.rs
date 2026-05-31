//! `tautology-or-contradiction` — flags binary comparisons that are always true or always false.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct TautologyOrContradictionDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// True if `expr` is a number Literal (kind == "number").
fn is_number_literal(expr: &Expression) -> bool {
    if let Expression::Literal(lit) = expr {
        return lit.kind.as_deref() == Some("number");
    }
    false
}

/// Return the numeric value string if the expression is a number literal, else None.
fn number_literal_value_str(expr: &Expression) -> Option<&str> {
    if let Expression::Literal(lit) = expr {
        if lit.kind.as_deref() == Some("number") {
            return lit.value.as_deref();
        }
    }
    None
}

/// True if the type string indicates an unsigned integer (starts with "uint").
fn is_unsigned_type(type_str: Option<&str>) -> bool {
    matches!(type_str, Some(t) if t.trim_start().starts_with("uint"))
}

impl IssueDetector for TautologyOrContradictionDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for binop in context.binary_operations() {
            let left = binop.left_expression.as_ref();
            let right = binop.right_expression.as_ref();
            let op = binop.operator.as_str();

            // Case 1: both operands are number literals — the result is a compile-time constant
            // and using it as a condition is always a tautology or contradiction.
            if is_number_literal(left) && is_number_literal(right) {
                if matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=") {
                    capture!(self, context, binop);
                    continue;
                }
            }

            // Case 2: unsigned integer compared against zero in a way that is trivially true/false.
            match op {
                // `uint_expr >= 0` — always true (unsigned can never be negative).
                ">=" => {
                    if let Some("0") = number_literal_value_str(right) {
                        if is_unsigned_type(left.type_string()) {
                            capture!(self, context, binop);
                        }
                    }
                    // `0 <= uint_expr` — symmetric always-true.
                    if let Some("0") = number_literal_value_str(left) {
                        if is_unsigned_type(right.type_string()) {
                            capture!(self, context, binop);
                        }
                    }
                }
                // `uint_expr < 0` — always false.
                "<" => {
                    if let Some("0") = number_literal_value_str(right) {
                        if is_unsigned_type(left.type_string()) {
                            capture!(self, context, binop);
                        }
                    }
                    // `0 > uint_expr` — expressed as the mirror: flag the `>` case below.
                }
                // `0 > uint_expr` — always false.
                ">" => {
                    if let Some("0") = number_literal_value_str(left) {
                        if is_unsigned_type(right.type_string()) {
                            capture!(self, context, binop);
                        }
                    }
                }
                // `uint_expr <= 0` only holds when uint == 0 ... but `0 >= uint_expr` is always
                // true, handled above.  `uint_expr <= 0` itself is NOT always false (it is true
                // when the value is 0), so we leave it unflagged to avoid false positives.
                _ => {}
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Tautology or Contradiction".to_string()
    }

    fn description(&self) -> String {
        "A comparison is always true or always false: remove the dead condition, fix the logic, \
         or use a signed integer type if negative values are intended."
            .to_string()
    }

    fn name(&self) -> String {
        "tautology-or-contradiction".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
