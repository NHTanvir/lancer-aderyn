//! `nested-struct-in-mapping` — flags structs that nest another struct containing a mapping.

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::error::Error;

use crate::ast::*;
use crate::capture;
use crate::context::{CaptureKey, WorkspaceContext};
use crate::detect::{IssueDetector, IssueSeverity};

#[derive(Default)]
pub struct NestedStructInMappingDetector {
    found_instances: BTreeMap<CaptureKey, NodeID>,
}

/// Returns true if `type_name` is or directly contains a `Mapping` node.
fn type_name_is_mapping(tn: &TypeName) -> bool {
    match tn {
        TypeName::Mapping(_) => true,
        TypeName::ArrayTypeName(a) => a.base_type.as_ref().map_or(false, |bt| type_name_is_mapping(bt)),
        _ => false,
    }
}

/// Returns true if the member's type_string contains "mapping(" (fallback text check).
fn type_string_contains_mapping(member: &VariableDeclaration) -> bool {
    member
        .type_descriptions
        .type_string
        .as_deref()
        .map_or(false, |ts| ts.contains("mapping("))
}

/// Returns true if the struct directly contains a mapping-typed member.
fn struct_directly_has_mapping(s: &StructDefinition) -> bool {
    s.members.iter().any(|m| {
        m.type_name
            .as_ref()
            .map_or(false, |tn| type_name_is_mapping(tn))
            || type_string_contains_mapping(m)
    })
}

/// Given a member's type_string (e.g. "struct MyContract.Inner"), extract the
/// bare struct name after the last dot (or after "struct ").
fn extract_struct_name_from_type_string(ts: &str) -> Option<&str> {
    // type_string looks like "struct ContractName.StructName" or "struct StructName"
    if !ts.contains("struct ") {
        return None;
    }
    // Take everything after "struct "
    let after = ts.split("struct ").nth(1)?;
    // If there's a dot, take the last component
    let bare = after.split('.').last()?;
    // Strip any trailing whitespace or brackets
    let bare = bare.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
    if bare.is_empty() {
        None
    } else {
        Some(bare)
    }
}

impl IssueDetector for NestedStructInMappingDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Step 1: collect IDs of structs that directly contain a mapping member.
        let mut mapping_struct_ids: HashSet<NodeID> = HashSet::new();
        let mut mapping_struct_names: HashSet<String> = HashSet::new();

        for s in context.struct_definitions() {
            if struct_directly_has_mapping(s) {
                mapping_struct_ids.insert(s.id);
                mapping_struct_names.insert(s.name.clone());
            }
        }

        // Step 2: fixed-point expansion — a struct containing a member whose type
        // refers to a mapping-struct is itself a mapping-struct (transitively).
        loop {
            let mut added = false;
            for s in context.struct_definitions() {
                if mapping_struct_ids.contains(&s.id) {
                    continue;
                }
                let has_nested = s.members.iter().any(|m| {
                    // Check via AST UserDefinedTypeName referenced_declaration
                    let via_ast = m.type_name.as_ref().map_or(false, |tn| {
                        if let TypeName::UserDefinedTypeName(udt) = tn {
                            udt.referenced_declaration
                                .map_or(false, |rid| mapping_struct_ids.contains(&rid))
                        } else {
                            false
                        }
                    });
                    // Fallback: check type_string for a known mapping-struct name
                    let via_type_string = m
                        .type_descriptions
                        .type_string
                        .as_deref()
                        .and_then(extract_struct_name_from_type_string)
                        .map_or(false, |name| mapping_struct_names.contains(name));
                    via_ast || via_type_string
                });
                if has_nested {
                    mapping_struct_ids.insert(s.id);
                    mapping_struct_names.insert(s.name.clone());
                    added = true;
                }
            }
            if !added {
                break;
            }
        }

        // Step 3: capture every struct that references a mapping-struct in one of
        // its members, but is NOT itself a direct-mapping struct (i.e., it nests
        // one). We flag the outer struct — the one that embeds the problematic type.
        for s in context.struct_definitions() {
            // Only flag structs that contain a member referring to a mapping-struct.
            // (We do not flag the innermost struct that only directly holds a mapping,
            // because that is the root cause definition; the real hazard is using it
            // as a nested member inside another struct.)
            let has_nested_mapping_struct = s.members.iter().any(|m| {
                let via_ast = m.type_name.as_ref().map_or(false, |tn| {
                    if let TypeName::UserDefinedTypeName(udt) = tn {
                        udt.referenced_declaration
                            .map_or(false, |rid| mapping_struct_ids.contains(&rid))
                    } else {
                        false
                    }
                });
                let via_type_string = m
                    .type_descriptions
                    .type_string
                    .as_deref()
                    .and_then(extract_struct_name_from_type_string)
                    .map_or(false, |name| mapping_struct_names.contains(name));
                via_ast || via_type_string
            });

            if has_nested_mapping_struct {
                capture!(self, context, s);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::High
    }

    fn title(&self) -> String {
        "Nested Struct in Mapping".to_string()
    }

    fn description(&self) -> String {
        "Avoid nesting structs that contain mappings inside other structs, as Solidity does not \
         support copying such composite types from storage to memory and the pattern can cause \
         silent data-corruption or revert; instead, flatten the data layout or store only \
         references (IDs) to the inner struct's mapping entries."
            .to_string()
    }

    fn name(&self) -> String {
        "nested-struct-in-mapping".to_string()
    }

    fn instances(&self) -> BTreeMap<CaptureKey, NodeID> {
        self.found_instances.clone()
    }
}
