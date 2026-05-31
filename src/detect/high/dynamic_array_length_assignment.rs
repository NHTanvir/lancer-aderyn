//! `dynamic-array-length-assignment` — flags direct assignments to a dynamic array's `.length`.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct DynamicArrayLengthAssignmentDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for DynamicArrayLengthAssignmentDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for a in context.assignments() {
            if let Expression::MemberAccess(ma) = a.left_hand_side.as_ref() {
                if ma.member_name == "length" {
                    capture!(self, context, a);
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Direct Assignment to Array Length".to_string()
    }

    fn description(&self) -> String {
        "Directly assigning to `array.length` is disallowed in Solidity >=0.6.0 and can corrupt \
         storage layout in older versions; use `push()` / `pop()` to resize arrays, or declare a \
         fixed-size array if the length is known at compile time."
            .to_string()
    }

    fn name(&self) -> String {
        "dynamic-array-length-assignment".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
