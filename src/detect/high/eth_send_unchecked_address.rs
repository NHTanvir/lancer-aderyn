//! `send-ether-no-checks` — ETH sent to an address that is not validated to be safe.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct EthSendUncheckedAddressDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Extract the "base" address expression from a member-access chain.
///
/// For `recipient.transfer(v)` the FunctionCall's expression is
/// `MemberAccess { expression: Identifier("recipient"), member_name: "transfer" }`.
/// We want the inner `Identifier("recipient")` as the base address.
///
/// For `addr.call{value: v}(...)` the FunctionCall's expression is a
/// `FunctionCallOptions` node; we recurse into its `expression` (which is a
/// MemberAccess) and peel one more level.
fn base_address_of_member_call(expr: &Expression) -> Option<&Expression> {
    match expr {
        // Direct member access: recipient.transfer / recipient.send / recipient.call
        Expression::MemberAccess(ma) => Some(ma.expression.as_ref()),
        // addr.call{value: v}  — the outer expression of the FunctionCallOptions
        // is itself a MemberAccess.
        Expression::FunctionCallOptions(fco) => {
            base_address_of_member_call(fco.expression.as_ref())
        }
        _ => None,
    }
}

/// Walk an address expression transitively stripping casts and member accesses
/// until we reach an `Identifier` or give up.
fn peel_to_identifier(expr: &Expression) -> Option<&Identifier> {
    match expr {
        Expression::Identifier(id) => Some(id),
        // address(someExpr) — type conversion
        Expression::FunctionCall(fc)
            if fc.kind.as_deref() == Some("typeConversion")
                && fc.arguments.len() == 1 =>
        {
            peel_to_identifier(&fc.arguments[0])
        }
        // (expr) — tuple wrap
        Expression::TupleExpression(te) if te.components.len() == 1 => {
            te.components[0].as_ref().and_then(peel_to_identifier)
        }
        _ => None,
    }
}

/// True when the identifier is `address(this)` — i.e. the identifier name is
/// "this".  (solc represents `this` as an `Identifier` with name "this".)
fn is_this(ident: &Identifier) -> bool {
    ident.name == "this"
}

/// Decide whether the resolved `VariableDeclaration` makes the address safe
/// (i.e. is a state variable or a constant).  If it is a parameter / local,
/// the address is considered attacker-controlled.
fn is_safe_variable(vd: &VariableDeclaration) -> bool {
    vd.state_variable || vd.is_constant() || vd.is_immutable()
}

/// Core check: given the expression used as the recipient in an ETH-sending
/// call, return true if the address is user-supplied (not safe).
fn recipient_is_unchecked(ctx: &WorkspaceContext, base: &Expression) -> bool {
    let ident = match peel_to_identifier(base) {
        Some(i) => i,
        // Cannot resolve — conservatively skip to avoid false positives.
        None => return false,
    };

    // `address(this)` is always safe.
    if is_this(ident) {
        return false;
    }

    match ident.referenced_declaration {
        None => false,
        Some(ref_id) => {
            // If the declaration resolves to a VariableDeclaration that is a
            // state variable / constant / immutable, it is considered safe.
            if let Some(vd) = ctx.resolve_variable(ref_id) {
                !is_safe_variable(vd)
            } else {
                // Could not resolve: skip.
                false
            }
        }
    }
}

impl IssueDetector for EthSendUncheckedAddressDetector {
    fn detect(&mut self, ctx: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // --- Pass 1: FunctionCall nodes -----------------------------------
        // Covers: recipient.transfer(amount) and recipient.send(amount)
        for call in ctx.function_calls() {
            // Only plain function calls (not type-conversions, struct constructors).
            if call.kind.as_deref() != Some("functionCall") {
                continue;
            }
            // The callee must be a MemberAccess with member "transfer" or "send".
            let ma = match call.expression.as_ref() {
                Expression::MemberAccess(ma) => ma,
                _ => continue,
            };
            if !matches!(ma.member_name.as_str(), "transfer" | "send") {
                continue;
            }
            let base = ma.expression.as_ref();
            if recipient_is_unchecked(ctx, base) {
                capture!(self, ctx, call);
            }
        }

        // --- Pass 2: FunctionCallOptions nodes ----------------------------
        // Covers: addr.call{value: v}(...)
        for fco in ctx.function_call_options() {
            // Must carry a "value" option.
            if !fco.names.iter().any(|n| n == "value") {
                continue;
            }
            // The inner expression must be a MemberAccess with member "call".
            let ma = match fco.expression.as_ref() {
                Expression::MemberAccess(ma) => ma,
                _ => continue,
            };
            if ma.member_name != "call" {
                continue;
            }
            let base = ma.expression.as_ref();
            if recipient_is_unchecked(ctx, base) {
                capture!(self, ctx, fco);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Sending ETH to Arbitrary Address".to_string()
    }

    fn description(&self) -> String {
        "ETH is transferred to an address that comes from a function parameter or local \
         variable without any ownership or allowlist check; validate the recipient against \
         a trusted set of addresses (e.g. store it as a state variable set in the constructor \
         or guard the function with an access-control modifier) before sending funds."
            .to_string()
    }

    fn name(&self) -> String {
        "send-ether-no-checks".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
