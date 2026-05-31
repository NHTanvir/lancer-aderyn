//! `centralization-risk` — flags functions guarded by privileged access-control modifiers.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::{has_modifier_like, is_public_or_external};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct CentralizationRiskDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for CentralizationRiskDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for f in context.function_definitions() {
            // Only flag public/external functions — internal helpers are less relevant.
            if !is_public_or_external(f) {
                continue;
            }
            // Catch common privileged-access modifier patterns:
            //   onlyOwner, onlyAdmin, onlyRole, onlyMinter, ...
            //   auth, authorized, ...
            //   hasRole, roleAdmin, ...
            //   ownerOnly, ...
            if has_modifier_like(f, "only")
                || has_modifier_like(f, "auth")
                || has_modifier_like(f, "role")
                || has_modifier_like(f, "owner")
            {
                capture!(self, context, f);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Centralization Risk".to_string()
    }

    fn description(&self) -> String {
        "Functions restricted to a single privileged address introduce a centralization risk; \
         consider replacing single-owner control with a multi-sig wallet, a timelock controller, \
         or a decentralised governance mechanism so that no single key can unilaterally alter \
         critical protocol state."
            .to_string()
    }

    fn name(&self) -> String {
        "centralization-risk".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
