//! `strict-equality-contract-balance` — flags strict equality checks against a `.balance` member access.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct StrictEqualityContractBalanceDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for StrictEqualityContractBalanceDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for binop in context.binary_operations() {
            if binop.operator != "==" && binop.operator != "!=" {
                continue;
            }
            let left_is_balance = matches!(
                binop.left_expression.as_ref(),
                Expression::MemberAccess(ma) if ma.member_name == "balance"
            );
            let right_is_balance = matches!(
                binop.right_expression.as_ref(),
                Expression::MemberAccess(ma) if ma.member_name == "balance"
            );
            if left_is_balance || right_is_balance {
                capture!(self, context, binop);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Strict Equality on Contract Balance".to_string()
    }

    fn description(&self) -> String {
        "Comparing a contract's Ether balance with strict equality (== or !=) is dangerous because \
         an attacker can forcibly send Ether via selfdestruct or coinbase assignment to break the \
         invariant; use >= or <= comparisons instead."
            .to_string()
    }

    fn name(&self) -> String {
        "strict-equality-contract-balance".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
