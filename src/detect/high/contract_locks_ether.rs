//! `contract-locks-ether` — flags contracts that can receive Ether but have no withdrawal path.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::{functions_of_contract, is_interface, is_library};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct ContractLocksEtherDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if the contract has any function that can receive Ether.
/// This means: a regular function with `payable` mutability, OR a `receive`
/// or `fallback` function that is payable.
fn contract_can_receive_ether(c: &ContractDefinition) -> bool {
    for f in functions_of_contract(c) {
        let kind = f.kind_str();
        let is_payable = f.mutability_str() == "payable";
        if is_payable {
            // Any payable function — regular, receive, or fallback — counts.
            return true;
        }
        // receive/fallback with no explicit mutability are nonpayable by default,
        // but we check again just in case solc emits them differently.
        if matches!(kind, "receive" | "fallback") && is_payable {
            return true;
        }
    }
    false
}

/// Returns true if there is evidence of a withdrawal path inside the contract:
/// - a `transfer` or `send` member access whose enclosing contract is `c`, OR
/// - a `FunctionCallOptions` with a `"value"` name key inside `c` (low-level
///   call with `{value: ...}`), OR
/// - a call to `selfdestruct` or `suicide` inside `c`.
fn contract_can_withdraw_ether(context: &WorkspaceContext, c: &ContractDefinition) -> bool {
    // Check for .transfer(...) or .send(...) member accesses scoped to this contract.
    for ma in context.member_accesses() {
        if matches!(ma.member_name.as_str(), "transfer" | "send") {
            if let Some(enc) = context.enclosing_contract(ma.id) {
                if enc.id == c.id {
                    return true;
                }
            }
        }
    }

    // Check for low-level call with {value: X} — FunctionCallOptions containing "value".
    for fco in context.function_call_options() {
        if fco.names.iter().any(|n| n == "value") {
            if let Some(enc) = context.enclosing_contract(fco.id) {
                if enc.id == c.id {
                    return true;
                }
            }
        }
    }

    // Check for selfdestruct(...) or suicide(...) calls.
    for call in context.function_calls() {
        let callee_name = match call.expression.as_ref() {
            Expression::Identifier(id) => Some(id.name.as_str()),
            _ => None,
        };
        if matches!(callee_name, Some("selfdestruct") | Some("suicide")) {
            if let Some(enc) = context.enclosing_contract(call.id) {
                if enc.id == c.id {
                    return true;
                }
            }
        }
    }

    false
}

impl IssueDetector for ContractLocksEtherDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for c in context.contract_definitions() {
            // Skip interfaces and libraries — only concrete contracts hold Ether.
            if is_interface(c) || is_library(c) {
                continue;
            }

            let can_receive = contract_can_receive_ether(c);
            let can_withdraw = contract_can_withdraw_ether(context, c);

            if can_receive && !can_withdraw {
                capture!(self, context, c);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Contract Locks Ether".to_string()
    }

    fn description(&self) -> String {
        "The contract accepts Ether via one or more payable functions but contains no code path \
         (transfer, send, low-level call with value, or selfdestruct) through which that Ether \
         can ever leave the contract; add a withdrawal function or remove the payable modifier \
         to prevent funds from being permanently locked."
            .to_string()
    }

    fn name(&self) -> String {
        "contract-locks-ether".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
