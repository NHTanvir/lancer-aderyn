//! `tx-origin-used-for-auth` — flags `tx.origin` used in an authorization check.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct TxOriginUsedForAuthDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Is `expr` the `tx.origin` member access?
fn is_tx_origin(expr: &Expression) -> bool {
    if let Expression::MemberAccess(ma) = expr {
        if ma.member_name == "origin" {
            if let Expression::Identifier(base) = ma.expression.as_ref() {
                return base.name == "tx";
            }
        }
    }
    false
}

impl IssueDetector for TxOriginUsedForAuthDetector {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // `tx.origin == X`, `X == tx.origin`, `tx.origin != X` in any comparison.
        for binop in ctx.binary_operations() {
            if !matches!(binop.operator.as_str(), "==" | "!=") {
                continue;
            }
            if is_tx_origin(&binop.left_expression) || is_tx_origin(&binop.right_expression) {
                capture!(self, ctx, binop);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Using `tx.origin` for Authorization".to_string()
    }

    fn description(&self) -> String {
        "`tx.origin` is the original externally-owned account that started the transaction; using \
         it for authorization makes the contract vulnerable to phishing-style attacks where a \
         victim is tricked into calling a malicious contract. Use `msg.sender` instead."
            .to_string()
    }

    fn name(&self) -> String {
        "tx-origin-used-for-auth".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
