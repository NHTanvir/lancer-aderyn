//! `immutable-state-variable` — a state variable written only inside the
//! constructor could be declared `immutable`.

use std::collections::{BTreeMap, HashSet};
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct StateVariableCouldBeImmutableDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

fn root_ref_id(expr: &Expression) -> Option<NodeID> {
    match expr {
        Expression::Identifier(id) => id.referenced_declaration,
        Expression::MemberAccess(m) => root_ref_id(&m.expression),
        Expression::IndexAccess(i) => root_ref_id(&i.base_expression),
        Expression::IndexRangeAccess(i) => root_ref_id(&i.base_expression),
        _ => None,
    }
}

impl IssueDetector for StateVariableCouldBeImmutableDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        let mut assigned_in_ctor: HashSet<NodeID> = HashSet::new();
        let mut assigned_outside: HashSet<NodeID> = HashSet::new();

        // (node_id, declaration_id) pairs for every write.
        let mut writes: Vec<(NodeID, NodeID)> = Vec::new();
        for a in context.assignments() {
            if let Some(id) = root_ref_id(&a.left_hand_side) {
                writes.push((a.id, id));
            }
        }
        for u in context.unary_operations() {
            if matches!(u.operator.as_str(), "++" | "--" | "delete") {
                if let Some(id) = root_ref_id(&u.sub_expression) {
                    writes.push((u.id, id));
                }
            }
        }
        for (node_id, decl) in writes {
            let in_ctor = context
                .enclosing_function(node_id)
                .map(|f| f.is_constructor())
                .unwrap_or(false);
            if in_ctor {
                assigned_in_ctor.insert(decl);
            } else {
                assigned_outside.insert(decl);
            }
        }

        for v in context.variable_declarations() {
            if v.state_variable
                && !v.is_constant()
                && !v.is_immutable()
                && assigned_in_ctor.contains(&v.id)
                && !assigned_outside.contains(&v.id)
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
        "State Variable Could Be Immutable".to_string()
    }

    fn description(&self) -> String {
        "This state variable is assigned only inside the constructor and never modified afterwards, \
         so it can be declared `immutable` to save gas on every read."
            .to_string()
    }

    fn name(&self) -> String {
        "immutable-state-variable".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
