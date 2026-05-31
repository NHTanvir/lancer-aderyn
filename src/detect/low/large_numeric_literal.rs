//! `large-numeric-literal` — flags bare integer literals with 7+ digits that lack scientific notation or underscores.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct LargeNumericLiteralDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for LargeNumericLiteralDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for lit in context.literals() {
            // Only plain number literals (not string/bool/hexString/unicodeString).
            if lit.kind.as_deref() != Some("number") {
                continue;
            }
            // Skip literals that carry a subdenomination (e.g. `1 ether`, `1 days`).
            if lit.subdenomination.is_some() {
                continue;
            }
            let value = match &lit.value {
                Some(v) => v,
                None => continue,
            };
            // Skip hex literals (e.g. 0x1234...).
            if value.starts_with("0x") || value.starts_with("0X") {
                continue;
            }
            // Strip underscores (Solidity allows 1_000_000) then count digits.
            let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
            if digits.len() >= 7 {
                capture!(self, context, lit);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Large Numeric Literal".to_string()
    }

    fn description(&self) -> String {
        "Replace large bare integer literals with scientific notation (e.g. `1e18`) or \
         underscore-separated groups (e.g. `1_000_000`) to improve readability and reduce \
         the risk of off-by-one-zero errors."
            .to_string()
    }

    fn name(&self) -> String {
        "large-numeric-literal".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
