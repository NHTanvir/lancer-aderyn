//! `state-variable-no-address-check` — setter assigns an address parameter to a
//! state variable without first checking it against the zero address.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::browser::ApproximateStorageChangeFinder;
use crate::context::flow;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::is_public_or_external;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct StateNoAddressCheckDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

fn identifier_ref(expr: &Expression) -> Option<NodeID> {
    match expr {
        Expression::Identifier(id) => id.referenced_declaration,
        _ => None,
    }
}

impl IssueDetector for StateNoAddressCheckDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        let finder = ApproximateStorageChangeFinder::new(context);
        for f in context.function_definitions() {
            if !is_public_or_external(f) {
                continue;
            }
            let Some(body) = &f.body else { continue };

            let addr_params: Vec<NodeID> = f
                .parameters
                .parameters
                .iter()
                .filter(|p| {
                    p.type_descriptions
                        .type_string
                        .as_deref()
                        .map(|t| t.starts_with("address"))
                        .unwrap_or(false)
                })
                .map(|p| p.id)
                .collect();
            if addr_params.is_empty() {
                continue;
            }

            let mut exprs = Vec::new();
            flow::expressions_in_statement(&Statement::Block(body.clone()), &mut exprs);

            // Params assigned directly into a state variable.
            let mut assigned_to_state: Vec<NodeID> = Vec::new();
            for e in &exprs {
                if let Expression::Assignment(a) = e {
                    if finder.target_is_state(&a.left_hand_side) {
                        if let Some(r) = identifier_ref(&a.right_hand_side) {
                            if addr_params.contains(&r) {
                                assigned_to_state.push(r);
                            }
                        }
                    }
                }
            }
            if assigned_to_state.is_empty() {
                continue;
            }

            // Params that are compared (== / !=) somewhere (a zero-address guard).
            let mut checked: Vec<NodeID> = Vec::new();
            for e in &exprs {
                if let Expression::BinaryOperation(b) = e {
                    if matches!(b.operator.as_str(), "==" | "!=") {
                        for side in [b.left_expression.as_ref(), b.right_expression.as_ref()] {
                            if let Some(r) = identifier_ref(side) {
                                if addr_params.contains(&r) {
                                    checked.push(r);
                                }
                            }
                        }
                    }
                }
            }

            if assigned_to_state.iter().any(|p| !checked.contains(p)) {
                capture!(self, context, f);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Missing Zero-Address Check in Setter".to_string()
    }

    fn description(&self) -> String {
        "This function assigns an address parameter to a state variable without checking it against \
         address(0). Add a `require(addr != address(0))` guard to prevent accidentally bricking the \
         contract."
            .to_string()
    }

    fn name(&self) -> String {
        "state-variable-no-address-check".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
