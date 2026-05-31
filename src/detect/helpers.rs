//! Reusable predicates shared across detectors (`is_state_variable`,
//! `function_visibility`, `has_modifier`, loop-ancestor checks, etc.).

use crate::ast::*;
use crate::context::WorkspaceContext;

/// Visibility string of a function ("public" / "external" / "internal" / "private").
pub fn function_visibility(f: &FunctionDefinition) -> &str {
    f.visibility_str()
}

pub fn is_public_or_external(f: &FunctionDefinition) -> bool {
    matches!(f.visibility_str(), "public" | "external")
}

pub fn is_internal_or_private(f: &FunctionDefinition) -> bool {
    matches!(f.visibility_str(), "internal" | "private")
}

/// True if the function is `view` or `pure` (cannot legitimately write state).
pub fn is_view_or_pure(f: &FunctionDefinition) -> bool {
    matches!(f.mutability_str(), "view" | "pure")
}

/// Does the function carry a modifier whose name matches `name`?
pub fn has_modifier(f: &FunctionDefinition, name: &str) -> bool {
    f.modifiers.iter().any(|m| {
        m.modifier_name
            .as_ref()
            .map(|p| p.name == name || p.name.ends_with(name))
            .unwrap_or(false)
    })
}

/// Case-insensitive substring match against any modifier name on `f`.
pub fn has_modifier_like(f: &FunctionDefinition, needle: &str) -> bool {
    let n = needle.to_lowercase();
    f.modifiers.iter().any(|m| {
        m.modifier_name
            .as_ref()
            .map(|p| p.name.to_lowercase().contains(&n))
            .unwrap_or(false)
    })
}

pub fn is_constructor(f: &FunctionDefinition) -> bool {
    f.is_constructor()
}

/// Is the variable a (mutable) state variable, i.e. storage that isn't constant?
pub fn is_state_variable(v: &VariableDeclaration) -> bool {
    v.state_variable
}

pub fn is_constant(v: &VariableDeclaration) -> bool {
    v.is_constant()
}

/// True if `node_id` is textually inside a loop (`for` / `while` / `do-while`).
pub fn is_inside_loop(ctx: &WorkspaceContext, node_id: NodeID) -> bool {
    let mut current = node_id;
    while let Some(parent) = ctx.get_parent(current) {
        match parent {
            AstNode::ForStatement(_)
            | AstNode::WhileStatement(_)
            | AstNode::DoWhileStatement(_) => return true,
            _ => {}
        }
        let next = parent.id();
        if next == current {
            break;
        }
        current = next;
    }
    false
}

/// The contract-kind ("contract" / "interface" / "library") of `c`.
pub fn contract_kind(c: &ContractDefinition) -> &str {
    c.contract_kind.as_deref().unwrap_or("contract")
}

pub fn is_interface(c: &ContractDefinition) -> bool {
    contract_kind(c) == "interface"
}

pub fn is_library(c: &ContractDefinition) -> bool {
    contract_kind(c) == "library"
}

/// Low-level call member names that move value or execute arbitrary code.
pub const LOW_LEVEL_CALL_MEMBERS: &[&str] = &["call", "delegatecall", "staticcall"];

/// Is this member access one of the low-level EVM call members?
pub fn is_low_level_call_member(member: &str) -> bool {
    LOW_LEVEL_CALL_MEMBERS.contains(&member)
}

/// Names that read block-level entropy (weak randomness sources).
pub const WEAK_RANDOMNESS_SOURCES: &[&str] =
    &["timestamp", "number", "difficulty", "prevrandao", "coinbase", "gaslimit", "basefee"];

/// True if a member access reads `block.<entropy>` or `blockhash(...)`.
pub fn is_block_entropy_member(member: &str) -> bool {
    WEAK_RANDOMNESS_SOURCES.contains(&member)
}

/// Extract the numeric value text from a `Literal`, if it is a number.
pub fn number_literal_value(l: &Literal) -> Option<&str> {
    if l.kind.as_deref() == Some("number") {
        l.value.as_deref()
    } else {
        None
    }
}

/// True if the type string denotes an address (`address` / `address payable`).
pub fn type_is_address(type_string: Option<&str>) -> bool {
    matches!(type_string, Some(t) if t.starts_with("address"))
}

/// Resolve the contract a function belongs to.
pub fn containing_contract<'a>(
    ctx: &'a WorkspaceContext,
    f: &FunctionDefinition,
) -> Option<&'a ContractDefinition> {
    ctx.enclosing_contract(f.id)
}

/// All functions textually declared in a contract.
pub fn functions_of_contract(c: &ContractDefinition) -> Vec<&FunctionDefinition> {
    c.nodes
        .iter()
        .filter_map(|n| match n {
            ContractBodyElement::FunctionDefinition(f) => Some(f),
            _ => None,
        })
        .collect()
}

/// All state-variable declarations textually declared in a contract.
pub fn state_variables_of_contract(c: &ContractDefinition) -> Vec<&VariableDeclaration> {
    c.nodes
        .iter()
        .filter_map(|n| match n {
            ContractBodyElement::VariableDeclaration(v) if v.state_variable => Some(v),
            _ => None,
        })
        .collect()
}

/// Pull the first pragma version literal string for a source unit, if any.
pub fn solidity_pragma_string(p: &PragmaDirective) -> String {
    p.literals.join("")
}
