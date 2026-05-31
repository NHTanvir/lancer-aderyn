//! `function-pointer-in-constructor` — flags constructors that accept function-pointer parameters.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct FunctionPointerInConstructorDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for FunctionPointerInConstructorDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for func in context.function_definitions() {
            if !func.is_constructor() {
                continue;
            }
            let mut flagged = false;
            'params: for param in &func.parameters.parameters {
                // Check if the TypeName is a FunctionTypeName variant.
                if let Some(TypeName::FunctionTypeName(_)) = &param.type_name {
                    flagged = true;
                    break 'params;
                }
                // Fallback: check type_descriptions.type_string for "function (".
                if let Some(ts) = param.type_descriptions.type_string.as_deref() {
                    if ts.contains("function (") {
                        flagged = true;
                        break 'params;
                    }
                }
            }
            if flagged {
                capture!(self, context, func);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Function Pointer Parameter in Constructor".to_string()
    }

    fn description(&self) -> String {
        "Passing function pointers as constructor arguments can lead to unpredictable behaviour \
         and makes contracts harder to audit; consider storing the target address instead and \
         calling the desired interface through a typed cast at call time."
            .to_string()
    }

    fn name(&self) -> String {
        "function-pointer-in-constructor".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
