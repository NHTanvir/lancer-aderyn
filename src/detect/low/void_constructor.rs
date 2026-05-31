//! `void-constructor` — a base constructor invoked with an empty argument list
//! (`Base()`), which is a no-op call.

use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct VoidConstructorDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

impl IssueDetector for VoidConstructorDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // `constructor() Base() {}` — base specifier on the constructor.
        for mi in context.modifier_invocations() {
            if mi.kind.as_deref() == Some("baseConstructorSpecifier") {
                if let Some(args) = &mi.arguments {
                    if args.is_empty() {
                        capture!(self, context, mi);
                    }
                }
            }
        }
        // `contract X is Base() {}` — empty-parens inheritance specifier.
        for spec in context.inheritance_specifiers() {
            if let Some(args) = &spec.arguments {
                if args.is_empty() {
                    capture!(self, context, spec);
                }
            }
        }
        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Void Constructor Call".to_string()
    }

    fn description(&self) -> String {
        "A base constructor is invoked with an empty argument list. If the base has no constructor \
         (or takes no arguments), the explicit `Base()` call is redundant and can be removed."
            .to_string()
    }

    fn name(&self) -> String {
        "void-constructor".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
