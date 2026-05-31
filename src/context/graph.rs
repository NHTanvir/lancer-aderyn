//! Inheritance + call graphs over the workspace (petgraph-backed).

use std::collections::HashMap;

use petgraph::graph::{DiGraph, NodeIndex};

use crate::ast::NodeID;
use crate::context::workspace_context::WorkspaceContext;

/// A directed graph keyed by AST `NodeID`s.
pub struct NodeGraph {
    pub graph: DiGraph<NodeID, ()>,
    pub index_of: HashMap<NodeID, NodeIndex>,
}

impl NodeGraph {
    fn empty() -> Self {
        NodeGraph { graph: DiGraph::new(), index_of: HashMap::new() }
    }

    fn ensure(&mut self, id: NodeID) -> NodeIndex {
        if let Some(ix) = self.index_of.get(&id) {
            *ix
        } else {
            let ix = self.graph.add_node(id);
            self.index_of.insert(id, ix);
            ix
        }
    }

    fn add_edge(&mut self, from: NodeID, to: NodeID) {
        let a = self.ensure(from);
        let b = self.ensure(to);
        self.graph.update_edge(a, b, ());
    }

    /// Direct successors of `id`.
    pub fn successors(&self, id: NodeID) -> Vec<NodeID> {
        let Some(&ix) = self.index_of.get(&id) else {
            return Vec::new();
        };
        self.graph
            .neighbors_directed(ix, petgraph::Direction::Outgoing)
            .map(|n| self.graph[n])
            .collect()
    }
}

impl WorkspaceContext {
    /// Build a contract-inheritance graph: edge child -> base.
    pub fn inheritance_graph(&self) -> NodeGraph {
        let mut g = NodeGraph::empty();
        for c in self.contract_definitions() {
            g.ensure(c.id);
            for &base in c.linearized_base_contracts.iter().skip(1) {
                g.add_edge(c.id, base);
            }
        }
        g
    }

    /// Build an approximate call graph: edge caller-fn -> callee-fn, resolved
    /// through `referenced_declaration` on the callee expression.
    pub fn call_graph(&self) -> NodeGraph {
        let mut g = NodeGraph::empty();
        for f in self.function_definitions() {
            g.ensure(f.id);
        }
        for call in self.function_calls() {
            let Some(callee) = referenced_callee(&call.expression) else {
                continue;
            };
            if let Some(caller) = self.enclosing_function(call.id) {
                g.add_edge(caller.id, callee);
            }
        }
        g
    }
}

fn referenced_callee(expr: &crate::ast::Expression) -> Option<NodeID> {
    use crate::ast::Expression::*;
    match expr {
        Identifier(id) => id.referenced_declaration,
        MemberAccess(m) => m.referenced_declaration,
        _ => None,
    }
}
