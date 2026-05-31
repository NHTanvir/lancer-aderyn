//! Browser helpers — the "navigate the indexed AST" operations that detectors
//! reach for: Peek (source extraction), ancestor/child walks, and the
//! ApproximateStorageChangeFinder used by the reentrancy detector.

use crate::ast::*;
use crate::context::workspace_context::WorkspaceContext;

impl WorkspaceContext {
    /// All direct children of `id` (nodes whose parent link points at `id`).
    pub fn immediate_children(&self, id: NodeID) -> Vec<&AstNode> {
        self.parent_link
            .iter()
            .filter(|(_, &p)| p == id)
            .filter_map(|(&c, _)| self.nodes.get(&c))
            .collect()
    }

    /// Resolve a `referenced_declaration` id to a `VariableDeclaration`.
    pub fn resolve_variable(&self, id: NodeID) -> Option<&VariableDeclaration> {
        match self.nodes.get(&id)? {
            AstNode::VariableDeclaration(v) => Some(v),
            _ => None,
        }
    }

    /// Resolve a `referenced_declaration` id to a `FunctionDefinition`.
    pub fn resolve_function(&self, id: NodeID) -> Option<&FunctionDefinition> {
        match self.nodes.get(&id)? {
            AstNode::FunctionDefinition(f) => Some(f),
            _ => None,
        }
    }

    /// Does `id` reference a state variable?
    pub fn references_state_variable(&self, referenced_declaration: Option<NodeID>) -> bool {
        referenced_declaration
            .and_then(|r| self.resolve_variable(r))
            .map(|v| v.state_variable)
            .unwrap_or(false)
    }
}

/// Approximate whether an expression mutates contract storage (a "state
/// change"). Heuristic, matching upstream's `ApproximateStorageChangeFinder`:
/// assignments / compound-assignments / `++` / `--` / `delete` whose target
/// resolves (transitively through member / index access) to a state variable.
pub struct ApproximateStorageChangeFinder<'a> {
    ctx: &'a WorkspaceContext,
}

impl<'a> ApproximateStorageChangeFinder<'a> {
    pub fn new(ctx: &'a WorkspaceContext) -> Self {
        Self { ctx }
    }

    /// True if `expr` (recursively) performs a state-variable mutation.
    pub fn expression_mutates_state(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Assignment(a) => {
                self.target_is_state(&a.left_hand_side)
                    || self.expression_mutates_state(&a.right_hand_side)
            }
            Expression::UnaryOperation(u) => {
                let op = u.operator.as_str();
                if (op == "++" || op == "--" || op == "delete")
                    && self.target_is_state(&u.sub_expression)
                {
                    return true;
                }
                self.expression_mutates_state(&u.sub_expression)
            }
            Expression::FunctionCall(f) => {
                f.arguments.iter().any(|a| self.expression_mutates_state(a))
            }
            Expression::Conditional(c) => {
                self.expression_mutates_state(&c.true_expression)
                    || self.expression_mutates_state(&c.false_expression)
            }
            Expression::TupleExpression(t) => t
                .components
                .iter()
                .flatten()
                .any(|c| self.expression_mutates_state(c)),
            _ => false,
        }
    }

    /// Whether an assignment target ultimately resolves to a state variable.
    pub fn target_is_state(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(id) => {
                self.ctx.references_state_variable(id.referenced_declaration)
            }
            Expression::MemberAccess(m) => {
                if let Some(r) = m.referenced_declaration {
                    if self.ctx.references_state_variable(Some(r)) {
                        return true;
                    }
                }
                self.target_is_state(&m.expression)
            }
            Expression::IndexAccess(i) => self.target_is_state(&i.base_expression),
            Expression::IndexRangeAccess(i) => self.target_is_state(&i.base_expression),
            Expression::TupleExpression(t) => {
                t.components.iter().flatten().any(|c| self.target_is_state(c))
            }
            _ => false,
        }
    }

    /// True if any statement in `block` mutates state.
    pub fn block_mutates_state(&self, block: &Block) -> bool {
        block.statements.iter().any(|s| self.statement_mutates_state(s))
    }

    pub fn statement_mutates_state(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::ExpressionStatement(e) => self.expression_mutates_state(&e.expression),
            Statement::Block(b) => self.block_mutates_state(b),
            Statement::UncheckedBlock(u) => {
                u.statements.iter().any(|s| self.statement_mutates_state(s))
            }
            Statement::IfStatement(i) => {
                self.statement_mutates_state(&i.true_body)
                    || i.false_body
                        .as_ref()
                        .is_some_and(|fb| self.statement_mutates_state(fb))
            }
            Statement::ForStatement(f) => self.statement_mutates_state(&f.body),
            Statement::WhileStatement(w) => self.statement_mutates_state(&w.body),
            Statement::DoWhileStatement(d) => self.statement_mutates_state(&d.body),
            Statement::VariableDeclarationStatement(v) => v
                .initial_value
                .as_ref()
                .is_some_and(|iv| self.expression_mutates_state(iv)),
            _ => false,
        }
    }
}
