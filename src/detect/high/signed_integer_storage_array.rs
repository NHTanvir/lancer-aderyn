//! `signed-storage-array` — flags storage arrays whose base type is a signed integer.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct SignedIntegerStorageArrayDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for SignedIntegerStorageArrayDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for v in context.variable_declarations() {
            if !v.state_variable {
                continue;
            }
            if let Some(TypeName::ArrayTypeName(arr)) = &v.type_name {
                if let Some(base) = &arr.base_type {
                    if let Some(ts) = base.type_string() {
                        // Match signed integer base types: "int", "int8" … "int256"
                        // but NOT "uint…" which also contains "int".
                        if ts.starts_with("int") && !ts.starts_with("uint") {
                            capture!(self, context, v);
                        }
                    }
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Signed Integer Storage Array".to_string()
    }

    fn description(&self) -> String {
        "Storage arrays with a signed integer base type can exhibit unexpected behaviour due to \
         Solidity's encoding of negative values; prefer unsigned integer arrays or add explicit \
         bounds checks to guard against underflow and unintended sign-extension."
            .to_string()
    }

    fn name(&self) -> String {
        "signed-storage-array".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
