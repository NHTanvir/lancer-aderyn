//! `state-variable-init-order` — a state variable initialized using another
//! state variable that is declared *after* it (an init-order dependency that
//! behaves differently under via-IR).

use std::collections::{BTreeMap, HashMap};
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::flow;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::helpers::state_variables_of_contract;
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct StateVariableInitOrderDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for StateVariableInitOrderDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for c in context.contract_definitions() {
            let svs = state_variables_of_contract(c);
            let index: HashMap<NodeID, usize> =
                svs.iter().enumerate().map(|(i, v)| (v.id, i)).collect();

            for (i, v) in svs.iter().enumerate() {
                let Some(value) = &v.value else { continue };
                let mut exprs = Vec::new();
                flow::walk_expression(value, &mut exprs);
                let refs_later = exprs.iter().any(|e| {
                    if let Expression::Identifier(id) = e {
                        id.referenced_declaration
                            .and_then(|r| index.get(&r))
                            .map(|&j| j > i)
                            .unwrap_or(false)
                    } else {
                        false
                    }
                });
                if refs_later {
                    capture!(self, context, *v);
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "State Variable Initialization Order Dependency".to_string()
    }

    fn description(&self) -> String {
        "This state variable's initializer reads another state variable that is declared later. \
         State variables are initialized in declaration order, so the referenced value will still \
         be zero; reorder the declarations or move the initialization into the constructor."
            .to_string()
    }

    fn name(&self) -> String {
        "state-variable-init-order".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
