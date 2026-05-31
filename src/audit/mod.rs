//! Auditor mode (`--auditor-mode`): instead of the normal detector output,
//! emit manual-audit scaffolds — curated listings an auditor uses to build a
//! scope index (external calls, low-level calls, delegatecalls, payable
//! functions, public/external entrypoints, assembly blocks, state variables).

use crate::ast::*;
use crate::context::WorkspaceContext;
use crate::detect::IssueSeverity;
use crate::report::{Issue, IssueInstance, Report};

struct Bucket {
    title: &'static str,
    description: &'static str,
    name: &'static str,
    instances: Vec<IssueInstance>,
}

impl Bucket {
    fn new(title: &'static str, description: &'static str, name: &'static str) -> Self {
        Bucket { title, description, name, instances: Vec::new() }
    }

    fn push(&mut self, ctx: &WorkspaceContext, src: &str, node_id: NodeID) {
        if let Some((file, line, snippet)) = ctx.capture_key(src) {
            self.instances.push(IssueInstance {
                contract_path: file,
                line_no: line,
                src: String::new(),
                snippet,
                node_id,
                hint: None,
            });
        }
    }

    fn into_issue(self) -> Option<Issue> {
        if self.instances.is_empty() {
            return None;
        }
        Some(Issue {
            title: self.title.to_string(),
            description: self.description.to_string(),
            detector_name: self.name.to_string(),
            severity: IssueSeverity::Low,
            instances: self.instances,
        })
    }
}

/// Build the auditor-mode report.
pub fn collect(ctx: &WorkspaceContext) -> Report {
    let mut external_calls = Bucket::new(
        "External Calls",
        "All external function calls — verify checks-effects-interactions.",
        "audit-external-calls",
    );
    let mut low_level = Bucket::new(
        "Low-level Calls",
        "All `call` / `delegatecall` / `staticcall` sites.",
        "audit-low-level-calls",
    );
    let mut delegatecalls = Bucket::new(
        "Delegate Calls",
        "All `delegatecall` sites — verify the target is trusted.",
        "audit-delegatecalls",
    );
    let mut payable_fns = Bucket::new(
        "Payable Functions",
        "All functions that can receive ETH.",
        "audit-payable-functions",
    );
    let mut entrypoints = Bucket::new(
        "Public / External Functions",
        "All externally-reachable entrypoints (attack surface).",
        "audit-entrypoints",
    );
    let mut assembly = Bucket::new(
        "Assembly Blocks",
        "All inline-assembly blocks — review memory / storage safety.",
        "audit-assembly",
    );
    let mut state_vars = Bucket::new(
        "State Variables",
        "All contract storage variables.",
        "audit-state-variables",
    );

    for ma in ctx.member_accesses() {
        match ma.member_name.as_str() {
            "call" | "staticcall" => low_level.push(ctx, &ma.src, ma.id),
            "delegatecall" => {
                low_level.push(ctx, &ma.src, ma.id);
                delegatecalls.push(ctx, &ma.src, ma.id);
            }
            _ => {}
        }
    }

    for f in ctx.function_definitions() {
        if f.mutability_str() == "payable" {
            payable_fns.push(ctx, &f.src, f.id);
        }
        if matches!(f.visibility_str(), "public" | "external") {
            entrypoints.push(ctx, &f.src, f.id);
        }
    }

    for asm in ctx.inline_assemblies() {
        assembly.push(ctx, &asm.src, asm.id);
    }

    for v in ctx.variable_declarations() {
        if v.state_variable {
            state_vars.push(ctx, &v.src, v.id);
        }
    }

    // Treat member-access calls that target known contracts as external.
    for call in ctx.function_calls() {
        if let Expression::MemberAccess(ma) = call.expression.as_ref() {
            if !matches!(ma.member_name.as_str(), "call" | "delegatecall" | "staticcall") {
                external_calls.push(ctx, &call.src, call.id);
            }
        }
    }

    let lows: Vec<Issue> = [
        external_calls,
        low_level,
        delegatecalls,
        payable_fns,
        entrypoints,
        assembly,
        state_vars,
    ]
    .into_iter()
    .filter_map(Bucket::into_issue)
    .collect();

    let files_summary = ctx
        .files
        .iter()
        .map(|f| crate::report::FileSummary {
            file_path: f.path.clone(),
            n_sloc: f.content.lines().filter(|l| !l.trim().is_empty()).count(),
        })
        .collect::<Vec<_>>();
    let total_sloc = files_summary.iter().map(|f| f.n_sloc).sum();

    Report {
        highs: Vec::new(),
        lows,
        files_summary,
        total_sloc,
        detectors_used: Vec::new(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}
