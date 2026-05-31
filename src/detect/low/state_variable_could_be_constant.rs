//! `constant-state-variable` — a state variable with an initializer that is
//! never reassigned could be declared `constant`.

use std::collections::{BTreeMap, HashSet};
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct StateVariableCouldBeConstantDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Resolve the root declaration id of an lvalue (`s.x`, `m[k]`, `a[i].b` -> the
/// base storage variable's declaration id).
fn root_ref_id(expr: &Expression) -> Option<NodeID> {
    match expr {
        Expression::Identifier(id) => id.referenced_declaration,
        Expression::MemberAccess(m) => root_ref_id(&m.expression),
        Expression::IndexAccess(i) => root_ref_id(&i.base_expression),
        Expression::IndexRangeAccess(i) => root_ref_id(&i.base_expression),
        _ => None,
    }
}

impl IssueDetector for StateVariableCouldBeConstantDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Every declaration id that is written anywhere.
        let mut assigned: HashSet<NodeID> = HashSet::new();
        for a in context.assignments() {
            if let Some(id) = root_ref_id(&a.left_hand_side) {
                assigned.insert(id);
            }
        }
        for u in context.unary_operations() {
            if matches!(u.operator.as_str(), "++" | "--" | "delete") {
                if let Some(id) = root_ref_id(&u.sub_expression) {
                    assigned.insert(id);
                }
            }
        }

        for v in context.variable_declarations() {
            if v.state_variable
                && !v.is_constant()
                && !v.is_immutable()
                && v.value.is_some()
                && !assigned.contains(&v.id)
            {
                capture!(self, context, v);
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "State Variable Could Be Constant".to_string()
    }

    fn description(&self) -> String {
        "This state variable is assigned an initial value and is never written to again, so it can \
         be declared `constant` to save gas (no storage slot is used)."
            .to_string()
    }

    fn name(&self) -> String {
        "constant-state-variable".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
