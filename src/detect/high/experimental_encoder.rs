//! `experimental-encoder` — flags use of `pragma experimental ABIEncoderV2`.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct ExperimentalEncoderDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for ExperimentalEncoderDetector {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for pragma in ctx.pragma_directives() {
            let has_experimental = pragma
                .literals
                .iter()
                .any(|l| l.eq_ignore_ascii_case("experimental"));
            let has_abi_encoder = pragma
                .literals
                .iter()
                .any(|l| l.to_ascii_lowercase().contains("abiencoder"));
            if has_experimental && has_abi_encoder {
                capture!(self, ctx, pragma);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Experimental ABIEncoderV2".to_string()
    }

    fn description(&self) -> String {
        "Remove `pragma experimental ABIEncoderV2` and upgrade to Solidity 0.8.0 or later, \
         where ABI encoder V2 is enabled by default and fully audited, to avoid known bugs \
         present in the experimental implementation."
            .to_string()
    }

    fn name(&self) -> String {
        "experimental-encoder".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
