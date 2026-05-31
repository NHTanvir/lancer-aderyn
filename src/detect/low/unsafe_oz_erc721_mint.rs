//! `unsafe-oz-erc721-mint` — using `_mint` instead of `_safeMint` for ERC721.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct UnsafeOzErc721MintDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for UnsafeOzErc721MintDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for call in context.function_calls() {
            let is_mint = match call.expression.as_ref() {
                Expression::Identifier(id) => id.name == "_mint",
                Expression::MemberAccess(m) => m.member_name == "_mint",
                _ => false,
            };
            if is_mint {
                capture!(self, context, call);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Unsafe ERC721 `_mint`".to_string()
    }

    fn description(&self) -> String {
        "`_mint` does not check whether the recipient can handle ERC721 tokens; use `_safeMint` so \
         that minting to a contract reverts unless it implements `onERC721Received`."
            .to_string()
    }

    fn name(&self) -> String {
        "unsafe-oz-erc721-mint".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
