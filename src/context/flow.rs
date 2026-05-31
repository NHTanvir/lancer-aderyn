//! Lightweight control-/data-flow primitives used by the heavier detectors
//! (reentrancy, costly-loop, delegatecall-in-loop).

use crate::ast::*;

/// Flatten a function body into a linear list of statements (depth-first,
/// preserving textual order). Nested blocks are inlined.
pub fn flatten_statements(stmt: &Statement, out: &mut Vec<Statement>) {
    out.push(stmt.clone());
    match stmt {
        Statement::Block(b) => {
            for s in &b.statements {
                flatten_statements(s, out);
            }
        }
        Statement::UncheckedBlock(u) => {
            for s in &u.statements {
                flatten_statements(s, out);
            }
        }
        Statement::IfStatement(i) => {
            flatten_statements(&i.true_body, out);
            if let Some(fb) = &i.false_body {
                flatten_statements(fb, out);
            }
        }
        Statement::ForStatement(f) => flatten_statements(&f.body, out),
        Statement::WhileStatement(w) => flatten_statements(&w.body, out),
        Statement::DoWhileStatement(d) => flatten_statements(&d.body, out),
        Statement::TryStatement(t) => {
            for c in &t.clauses {
                flatten_statements(&Statement::Block(c.block.clone()), out);
            }
        }
        _ => {}
    }
}

/// Collect every expression appearing anywhere under `expr` (inclusive).
pub fn walk_expression(expr: &Expression, out: &mut Vec<Expression>) {
    out.push(expr.clone());
    match expr {
        Expression::Assignment(a) => {
            walk_expression(&a.left_hand_side, out);
            walk_expression(&a.right_hand_side, out);
        }
        Expression::BinaryOperation(b) => {
            walk_expression(&b.left_expression, out);
            walk_expression(&b.right_expression, out);
        }
        Expression::UnaryOperation(u) => walk_expression(&u.sub_expression, out),
        Expression::FunctionCall(f) => {
            walk_expression(&f.expression, out);
            for a in &f.arguments {
                walk_expression(a, out);
            }
        }
        Expression::FunctionCallOptions(f) => {
            walk_expression(&f.expression, out);
            for o in &f.options {
                walk_expression(o, out);
            }
        }
        Expression::MemberAccess(m) => walk_expression(&m.expression, out),
        Expression::IndexAccess(i) => {
            walk_expression(&i.base_expression, out);
            if let Some(ie) = &i.index_expression {
                walk_expression(ie, out);
            }
        }
        Expression::Conditional(c) => {
            walk_expression(&c.condition, out);
            walk_expression(&c.true_expression, out);
            walk_expression(&c.false_expression, out);
        }
        Expression::TupleExpression(t) => {
            for c in t.components.iter().flatten() {
                walk_expression(c, out);
            }
        }
        _ => {}
    }
}

/// Collect every expression appearing in a statement subtree.
pub fn expressions_in_statement(stmt: &Statement, out: &mut Vec<Expression>) {
    let mut stmts = Vec::new();
    flatten_statements(stmt, &mut stmts);
    for s in &stmts {
        match s {
            Statement::ExpressionStatement(e) => walk_expression(&e.expression, out),
            Statement::IfStatement(i) => walk_expression(&i.condition, out),
            Statement::WhileStatement(w) => walk_expression(&w.condition, out),
            Statement::DoWhileStatement(d) => walk_expression(&d.condition, out),
            Statement::ForStatement(f) => {
                if let Some(c) = &f.condition {
                    walk_expression(c, out);
                }
            }
            Statement::Return(r) => {
                if let Some(e) = &r.expression {
                    walk_expression(e, out);
                }
            }
            Statement::VariableDeclarationStatement(v) => {
                if let Some(iv) = &v.initial_value {
                    walk_expression(iv, out);
                }
            }
            Statement::EmitStatement(e) => {
                walk_expression(&Expression::FunctionCall(e.event_call.clone()), out)
            }
            Statement::RevertStatement(r) => {
                walk_expression(&Expression::FunctionCall(r.error_call.clone()), out)
            }
            _ => {}
        }
    }
}
