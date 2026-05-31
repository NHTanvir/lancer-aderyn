//! `unprotected-init-solmate` — flags solmate SafeTransferLib usage that skips code-existence checks.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

/// The set of solmate SafeTransferLib member names that silently succeed on
/// addresses with no code (e.g. undeployed tokens).
const SAFE_TRANSFER_MEMBERS: &[&str] = &[
    "safeTransfer",
    "safeTransferFrom",
    "safeApprove",
    "safeTransferETH",
];

#[derive(Default)]
pub struct SolmateSafeTransferLibDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for SolmateSafeTransferLibDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Only flag calls when solmate's SafeTransferLib is actually imported via a
        // `using SafeTransferLib for …` directive somewhere in the workspace.
        let uses_solmate = context.using_for_directives().iter().any(|ufd| {
            ufd.library_name
                .as_ref()
                .map(|ip| ip.name.contains("SafeTransferLib"))
                .unwrap_or(false)
        });

        if !uses_solmate {
            return Ok(false);
        }

        // Capture every function call whose callee is a member access to one of
        // the dangerous SafeTransferLib methods.
        for call in context.function_calls() {
            if let Expression::MemberAccess(ma) = call.expression.as_ref() {
                if SAFE_TRANSFER_MEMBERS.contains(&ma.member_name.as_str()) {
                    capture!(self, context, call);
                }
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Solmate SafeTransferLib Misuse".to_string()
    }

    fn description(&self) -> String {
        "solmate's SafeTransferLib does not verify that the token address contains contract code, \
         so calls to safeTransfer/safeTransferFrom/safeApprove/safeTransferETH silently succeed \
         on non-existent or self-destructed token contracts; replace with OpenZeppelin's \
         SafeERC20 or add an explicit code-existence check before transferring tokens."
            .to_string()
    }

    fn name(&self) -> String {
        "unprotected-init-solmate".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
