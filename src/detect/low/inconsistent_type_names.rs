//! `inconsistent-type-names` — flags alias type names (uint, int, byte) when
//! both the alias and a canonical form (e.g. uint256, int256, bytes1) appear in
//! the same workspace.

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct InconsistentTypeNamesDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Return the base alias group for an ElementaryTypeName, if it belongs to one
/// of the alias families we track.
///
/// - "uint"  / "uint<N>"  -> Some("uint")
/// - "int"   / "int<N>"   -> Some("int")
/// - "byte"  / "bytes<N>" -> Some("byte")
///
/// Any other name returns `None`.
fn alias_group(name: &str) -> Option<&'static str> {
    if name == "uint" || (name.starts_with("uint") && name.len() > 4) {
        Some("uint")
    } else if name == "int" || (name.starts_with("int") && name.len() > 3) {
        Some("int")
    } else if name == "byte" || name.starts_with("bytes") {
        Some("byte")
    } else {
        None
    }
}

/// True if this name is the un-suffixed alias form (not the canonical form).
fn is_alias(name: &str) -> bool {
    name == "uint" || name == "int" || name == "byte"
}

/// Collect the `name` field from every `ElementaryTypeName` that appears as the
/// `type_name` of a `VariableDeclaration`.
fn collect_elem_type_name(vd: &VariableDeclaration) -> Option<&str> {
    match vd.type_name.as_ref()? {
        TypeName::ElementaryTypeName(etn) => Some(&etn.name),
        _ => None,
    }
}

impl IssueDetector for InconsistentTypeNamesDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // --- Pass 1: gather every distinct elementary type name used across
        //             all variable declarations.
        let mut seen_groups: HashSet<&'static str> = HashSet::new();
        let mut has_alias: HashSet<&'static str> = HashSet::new();
        let mut has_canonical: HashSet<&'static str> = HashSet::new();

        for vd in context.variable_declarations() {
            if let Some(name) = collect_elem_type_name(vd) {
                if let Some(group) = alias_group(name) {
                    seen_groups.insert(group);
                    if is_alias(name) {
                        has_alias.insert(group);
                    } else {
                        has_canonical.insert(group);
                    }
                }
            }
        }

        // Groups where BOTH alias and canonical forms are present.
        let mixed: HashSet<&'static str> = seen_groups
            .into_iter()
            .filter(|g| has_alias.contains(g) && has_canonical.contains(g))
            .collect();

        if mixed.is_empty() {
            return Ok(false);
        }

        // --- Pass 2: capture every variable declaration that uses the alias
        //             form in a mixed group.
        for vd in context.variable_declarations() {
            if let Some(name) = collect_elem_type_name(vd) {
                if is_alias(name) {
                    if let Some(group) = alias_group(name) {
                        if mixed.contains(group) {
                            capture!(self, context, vd);
                        }
                    }
                }
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        "Inconsistent Type Names (uint vs uint256)".to_string()
    }

    fn description(&self) -> String {
        "The codebase mixes alias type names (e.g. `uint`, `int`, `byte`) with their canonical \
         equivalents (e.g. `uint256`, `int256`, `bytes1`); standardise on the explicit canonical \
         forms throughout to improve readability and avoid confusion."
            .to_string()
    }

    fn name(&self) -> String {
        "inconsistent-type-names".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
