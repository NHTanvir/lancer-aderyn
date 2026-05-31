//! `unsafe-erc20-functions` — a raw IERC20 `transfer` / `transferFrom` /
//! `approve` call that doesn't go through SafeERC20.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct UnsafeErc20OperationDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for UnsafeErc20OperationDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for call in context.function_calls() {
            if let Expression::MemberAccess(ma) = call.expression.as_ref() {
                if matches!(ma.member_name.as_str(), "transfer" | "transferFrom" | "approve") {
                    let base_ts = ma.expression.type_string().unwrap_or("");
                    if base_ts.contains("ERC20") {
                        capture!(self, context, call);
                    }
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Unsafe ERC20 Operation".to_string()
    }

    fn description(&self) -> String {
        "This calls a raw IERC20 transfer/transferFrom/approve. Some tokens do not return a bool \
         (or return false instead of reverting); use OpenZeppelin's SafeERC20 (safeTransfer, etc.)."
            .to_string()
    }

    fn name(&self) -> String {
        "unsafe-erc20-functions".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
