//! `unspecific-solidity-pragma` — flags floating pragmas like `^0.8.0`.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct UnspecificSolidityPragmaDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for UnspecificSolidityPragmaDetector {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for pragma in ctx.pragma_directives() {
            // Only consider solidity version pragmas.
            if pragma.literals.first().map(|s| s.as_str()) != Some("solidity") {
                continue;
            }
            let joined = pragma.literals.join("");
            // Floating if it carries a caret/tilde or comparison operator.
            if joined.contains('^') || joined.contains('~') || joined.contains('>') || joined.contains('<') {
                capture!(self, ctx, pragma);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Unspecific Solidity Pragma".to_string()
    }

    fn description(&self) -> String {
        "Consider using a specific version of Solidity in your contracts instead of a wide \
         version. For example, instead of `pragma solidity ^0.8.0;`, use `pragma solidity 0.8.0;`."
            .to_string()
    }

    fn name(&self) -> String {
        "unspecific-solidity-pragma".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
