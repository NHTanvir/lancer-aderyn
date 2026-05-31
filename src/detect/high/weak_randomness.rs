//! `weak-randomness` — flags block-level entropy used as a randomness source.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::is_block_entropy_member;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct WeakRandomnessDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for WeakRandomnessDetector {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // `block.timestamp`, `block.prevrandao`, `block.number`, ...
        for ma in ctx.member_accesses() {
            if let Expression::Identifier(base) = ma.expression.as_ref() {
                if base.name == "block" && is_block_entropy_member(&ma.member_name) {
                    capture!(self, ctx, ma);
                }
            }
        }
        // `blockhash(...)`
        for call in ctx.function_calls() {
            if let Expression::Identifier(id) = call.expression.as_ref() {
                if id.name == "blockhash" {
                    capture!(self, ctx, call);
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Weak Randomness".to_string()
    }

    fn description(&self) -> String {
        "The use of keccak256 hash functions on predictable values like block.timestamp, \
         block.number, blockhash, or block.prevrandao can be exploited by miners/validators to \
         manipulate outcomes. Use a verifiable randomness source such as Chainlink VRF."
            .to_string()
    }

    fn name(&self) -> String {
        "weak-randomness".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
