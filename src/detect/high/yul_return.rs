//! `yul-return` — Yul `return` Bypasses Modifiers.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct YulReturnDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Recursively scan a Yul AST JSON value for a `YulFunctionCall` whose
/// `functionName.name` is `"return"`.
fn scan(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            // Check if this object is a YulFunctionCall named "return".
            if map.get("nodeType").and_then(|v| v.as_str()) == Some("YulFunctionCall") {
                if let Some(fn_name) = map.get("functionName") {
                    if fn_name.get("name").and_then(|v| v.as_str()) == Some("return") {
                        return true;
                    }
                }
            }
            // Recurse into all values of the object.
            map.values().any(scan)
        }
        serde_json::Value::Array(arr) => arr.iter().any(scan),
        _ => false,
    }
}

impl IssueDetector for YulReturnDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for asm in context.inline_assemblies() {
            if let Some(yul_ast) = &asm.ast {
                if scan(yul_ast) {
                    capture!(self, context, asm);
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Yul `return` Bypasses Modifiers".to_string()
    }

    fn description(&self) -> String {
        "A Yul `return` opcode exits the entire call frame immediately, skipping any \
         post-execution modifier logic (e.g., reentrancy guards, state finalization); \
         replace inline-assembly `return` with normal Solidity `return` statements or \
         restructure the assembly block so that modifier code still executes."
            .to_string()
    }

    fn name(&self) -> String {
        "yul-return".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
