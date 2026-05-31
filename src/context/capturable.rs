//! The `Capturable` trait — anything a detector can `capture!`.
//!
//! It unifies concrete AST structs (via the `Node` trait), the `Expression`
//! enum, the unified `AstNode`, and references to all of these, so the
//! `capture!` macro accepts any of them.

use crate::ast::{AstNode, Expression, Node, NodeID};

pub trait Capturable {
    fn capturable_src(&self) -> &str;
    fn capturable_id(&self) -> NodeID;
}

impl<T: Node> Capturable for T {
    fn capturable_src(&self) -> &str {
        self.src()
    }
    fn capturable_id(&self) -> NodeID {
        self.id()
    }
}

// NOTE: we deliberately do NOT add a blanket `impl<T: Capturable> Capturable
// for &T` — that would conflict (E0119) with the `impl<T: Node>` blanket above.
// The `capture!` macro instead relies on method-call auto-deref, so passing a
// `&BinaryOperation` (or `&&BinaryOperation`) resolves fine through these impls.

impl Capturable for Expression {
    fn capturable_src(&self) -> &str {
        self.src()
    }
    fn capturable_id(&self) -> NodeID {
        self.id()
    }
}

impl Capturable for AstNode {
    fn capturable_src(&self) -> &str {
        self.src()
    }
    fn capturable_id(&self) -> NodeID {
        self.id()
    }
}
