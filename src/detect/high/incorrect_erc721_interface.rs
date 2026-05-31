//! `incorrect-erc721-interface` — flags ERC721 functions whose parameter or return types
//! deviate from the EIP-721 specification.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct IncorrectErc721InterfaceDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

// ---------------------------------------------------------------------------
// Helpers — extract the raw elementary type name from a TypeName node.
// ---------------------------------------------------------------------------

/// Canonicalise a solc type-string to the bare elementary name we compare
/// against ("uint256", "address", "bool", "bytes").  Returns `None` for
/// unknown / complex types.
fn elementary_name(tn: &TypeName) -> Option<&str> {
    match tn {
        TypeName::ElementaryTypeName(e) => Some(e.name.as_str()),
        _ => {
            // Fall back to the type-description string produced by solc.
            let ts = tn.type_string()?;
            // Strip trailing " payable" and location qualifiers.
            let ts = ts.trim();
            let ts = ts.strip_suffix(" payable").unwrap_or(ts);
            Some(ts)
        }
    }
}

/// Collect the elementary names of all parameters in a `ParameterList`.
/// Returns `None` if any parameter has a type we cannot resolve.
fn param_types(pl: &ParameterList) -> Option<Vec<&str>> {
    pl.parameters
        .iter()
        .map(|p| p.type_name.as_ref().and_then(|t| elementary_name(t)))
        .collect()
}

// ---------------------------------------------------------------------------
// Per-function ERC721 signature checks
// ---------------------------------------------------------------------------

/// Returns `true` if the function's signature matches the ERC721 spec.
/// Returns `false` if the name matched but the signature is wrong (= finding).
/// Returns `None` if the function name is not an ERC721 method (skip).
fn erc721_signature_ok(f: &FunctionDefinition) -> Option<bool> {
    let params = param_types(&f.parameters)?;
    let returns = param_types(&f.return_parameters)?;

    let ok = match f.name.as_str() {
        // balanceOf(address owner) -> uint256
        "balanceOf" => {
            params.len() == 1
                && params[0].starts_with("address")
                && returns.len() == 1
                && returns[0] == "uint256"
        }

        // ownerOf(uint256 tokenId) -> address
        "ownerOf" => {
            params.len() == 1
                && params[0] == "uint256"
                && returns.len() == 1
                && returns[0].starts_with("address")
        }

        // safeTransferFrom(address,address,uint256)  — no bytes variant forced
        // safeTransferFrom(address,address,uint256,bytes)  — also fine
        "safeTransferFrom" => {
            let base_ok = params.len() >= 3
                && params[0].starts_with("address")
                && params[1].starts_with("address")
                && params[2] == "uint256";
            let extra_ok = if params.len() == 4 {
                // fourth param must be bytes (memory)
                params[3].starts_with("bytes")
            } else {
                params.len() == 3
            };
            base_ok && extra_ok && returns.is_empty()
        }

        // transferFrom(address,address,uint256) -> ()
        "transferFrom" => {
            params.len() == 3
                && params[0].starts_with("address")
                && params[1].starts_with("address")
                && params[2] == "uint256"
                && returns.is_empty()
        }

        // approve(address,uint256) -> ()
        "approve" => {
            params.len() == 2
                && params[0].starts_with("address")
                && params[1] == "uint256"
                && returns.is_empty()
        }

        // setApprovalForAll(address,bool) -> ()
        "setApprovalForAll" => {
            params.len() == 2
                && params[0].starts_with("address")
                && params[1] == "bool"
                && returns.is_empty()
        }

        // getApproved(uint256) -> address
        "getApproved" => {
            params.len() == 1
                && params[0] == "uint256"
                && returns.len() == 1
                && returns[0].starts_with("address")
        }

        // isApprovedForAll(address,address) -> bool
        "isApprovedForAll" => {
            params.len() == 2
                && params[0].starts_with("address")
                && params[1].starts_with("address")
                && returns.len() == 1
                && returns[0] == "bool"
        }

        // Not an ERC721 method — skip.
        _ => return None,
    };

    Some(ok)
}

// ---------------------------------------------------------------------------
// IssueDetector implementation
// ---------------------------------------------------------------------------

impl IssueDetector for IncorrectErc721InterfaceDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for f in context.function_definitions() {
            // Only consider regular named functions (not constructors etc.).
            if f.kind_str() != "function" {
                continue;
            }
            // Only public / external — ERC721 interface functions must be accessible.
            match f.visibility_str() {
                "public" | "external" => {}
                _ => continue,
            }
            // Check against ERC721 spec; capture when the name matches but sig is wrong.
            if let Some(false) = erc721_signature_ok(f) {
                capture!(self, context, f);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Incorrect ERC721 Interface".to_string()
    }

    fn description(&self) -> String {
        "One or more functions share a name with an ERC721 standard method but use different \
         parameter or return types; ensure all ERC721 functions exactly match the EIP-721 \
         specification so that wallets, marketplaces, and other contracts can interact with \
         the token correctly."
            .to_string()
    }

    fn name(&self) -> String {
        "incorrect-erc721-interface".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
