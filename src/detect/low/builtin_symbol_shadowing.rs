//! `builtin-symbol-shadowing` — flags declarations that shadow a Solidity builtin symbol.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

/// Solidity builtin identifiers that must not be redeclared.
const BUILTINS: &[&str] = &[
    "now",
    "revert",
    "assert",
    "require",
    "keccak256",
    "sha256",
    "sha3",
    "ecrecover",
    "addmod",
    "mulmod",
    "selfdestruct",
    "suicide",
    "blockhash",
    "gasleft",
    "this",
    "super",
    "msg",
    "tx",
    "block",
    "abi",
    "type",
    "fallback",
    "receive",
];

#[derive(Default)]
pub struct BuiltinSymbolShadowingDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for BuiltinSymbolShadowingDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Variable declarations (state variables, local variables, parameters)
        for v in context.variable_declarations() {
            if BUILTINS.contains(&v.name.as_str()) {
                capture!(self, context, v);
            }
        }

        // Function definitions
        for f in context.function_definitions() {
            if BUILTINS.contains(&f.name.as_str()) {
                capture!(self, context, f);
            }
        }

        // Modifier definitions
        for m in context.modifier_definitions() {
            if BUILTINS.contains(&m.name.as_str()) {
                capture!(self, context, m);
            }
        }

        // Event definitions
        for e in context.event_definitions() {
            if BUILTINS.contains(&e.name.as_str()) {
                capture!(self, context, e);
            }
        }

        // Struct definitions
        for s in context.struct_definitions() {
            if BUILTINS.contains(&s.name.as_str()) {
                capture!(self, context, s);
            }
        }

        // Enum definitions
        for e in context.enum_definitions() {
            if BUILTINS.contains(&e.name.as_str()) {
                capture!(self, context, e);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Shadowing a Builtin Symbol".to_string()
    }

    fn description(&self) -> String {
        "Rename the declaration so it does not collide with a Solidity builtin (e.g. \
         `require`, `keccak256`, `msg`, `block`); shadowing builtins makes the code \
         misleading and can silently break expected language semantics."
            .to_string()
    }

    fn name(&self) -> String {
        "builtin-symbol-shadowing".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
