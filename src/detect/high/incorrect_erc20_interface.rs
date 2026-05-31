//! `incorrect-erc20-interface` — flags ERC-20 functions whose signatures deviate from the standard.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::is_public_or_external;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct IncorrectErc20InterfaceDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Return the `type_string` of the sole return parameter, if there is exactly one.
fn single_return_type(f: &FunctionDefinition) -> Option<&str> {
    let params = &f.return_parameters.parameters;
    if params.len() == 1 {
        params[0].type_descriptions.type_string.as_deref()
    } else {
        None
    }
}

/// Return true when the function has no return parameters at all.
fn returns_nothing(f: &FunctionDefinition) -> bool {
    f.return_parameters.parameters.is_empty()
}

/// Collect the `type_string` of each input parameter in order.
fn param_type_strings(f: &FunctionDefinition) -> Vec<Option<&str>> {
    f.parameters
        .parameters
        .iter()
        .map(|p| p.type_descriptions.type_string.as_deref())
        .collect()
}

/// True when `ts` equals "bool" (exact match).
fn is_bool(ts: Option<&str>) -> bool {
    ts == Some("bool")
}

/// True when `ts` equals "uint256" (exact match).
fn is_uint256(ts: Option<&str>) -> bool {
    ts == Some("uint256")
}

/// True when `ts` starts with "address" (covers `address payable` too).
fn is_address(ts: Option<&str>) -> bool {
    matches!(ts, Some(t) if t.starts_with("address"))
}

// ---------------------------------------------------------------------------
// Detection logic
// ---------------------------------------------------------------------------

/// Check whether `f` violates ERC-20 interface requirements and should be flagged.
///
/// Expected signatures:
///   transfer(address, uint256) -> bool
///   approve(address, uint256) -> bool
///   transferFrom(address, address, uint256) -> bool
///   balanceOf(address) -> uint256
///   totalSupply() -> uint256
///   allowance(address, address) -> uint256
fn is_incorrect_erc20(f: &FunctionDefinition) -> bool {
    let pts = param_type_strings(f);

    match f.name.as_str() {
        // ---- functions that MUST return bool --------------------------------
        "transfer" => {
            // Expected: (address, uint256) -> bool
            let params_ok = pts.len() == 2 && is_address(pts[0]) && is_uint256(pts[1]);
            if !params_ok {
                // Only flag when the param count/types match the intent so we
                // avoid hitting completely unrelated overloads; flag return mismatch
                // regardless.
                return false;
            }
            returns_nothing(f) || !is_bool(single_return_type(f))
        }

        "approve" => {
            // Expected: (address, uint256) -> bool
            let params_ok = pts.len() == 2 && is_address(pts[0]) && is_uint256(pts[1]);
            if !params_ok {
                return false;
            }
            returns_nothing(f) || !is_bool(single_return_type(f))
        }

        "transferFrom" => {
            // Expected: (address, address, uint256) -> bool
            let params_ok =
                pts.len() == 3 && is_address(pts[0]) && is_address(pts[1]) && is_uint256(pts[2]);
            if !params_ok {
                return false;
            }
            returns_nothing(f) || !is_bool(single_return_type(f))
        }

        // ---- functions that MUST return uint256 -----------------------------
        "balanceOf" => {
            // Expected: (address) -> uint256
            let params_ok = pts.len() == 1 && is_address(pts[0]);
            if !params_ok {
                return false;
            }
            returns_nothing(f) || !is_uint256(single_return_type(f))
        }

        "totalSupply" => {
            // Expected: () -> uint256
            let params_ok = pts.is_empty();
            if !params_ok {
                return false;
            }
            returns_nothing(f) || !is_uint256(single_return_type(f))
        }

        "allowance" => {
            // Expected: (address, address) -> uint256
            let params_ok = pts.len() == 2 && is_address(pts[0]) && is_address(pts[1]);
            if !params_ok {
                return false;
            }
            returns_nothing(f) || !is_uint256(single_return_type(f))
        }

        _ => false,
    }
}

impl IssueDetector for IncorrectErc20InterfaceDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for f in context.function_definitions() {
            if !is_public_or_external(f) {
                continue;
            }
            if is_incorrect_erc20(f) {
                capture!(self, context, f);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Incorrect ERC20 Interface".to_string()
    }

    fn description(&self) -> String {
        "One or more ERC-20 functions (transfer, approve, transferFrom, balanceOf, totalSupply, \
         allowance) do not match the return type mandated by EIP-20; ensure each function returns \
         the correct type (bool for transfer/approve/transferFrom, uint256 for \
         balanceOf/totalSupply/allowance) so that callers and DEX routers that rely on the standard \
         ABI behave correctly."
            .to_string()
    }

    fn name(&self) -> String {
        "incorrect-erc20-interface".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
