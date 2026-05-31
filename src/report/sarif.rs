//! SARIF 2.1.0 report emitter (for GitHub code-scanning upload).
//!
//! We construct the SARIF document directly with `serde_json` to stay schema-
//! faithful without pinning the exact API of an external SARIF crate.

use serde_json::{json, Value};

use super::{Issue, RenderOptions, Report};
use crate::detect::IssueSeverity;

const SARIF_SCHEMA: &str =
    "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json";

fn level_for(sev: IssueSeverity) -> &'static str {
    match sev {
        IssueSeverity::High => "error",
        IssueSeverity::Low => "warning",
    }
}

fn rules_and_results(report: &Report, opts: RenderOptions) -> (Vec<Value>, Vec<Value>) {
    let mut rules = Vec::new();
    let mut results = Vec::new();

    let issues: Vec<&Issue> = if opts.highs_only {
        report.highs.iter().collect()
    } else {
        report.highs.iter().chain(report.lows.iter()).collect()
    };

    for issue in issues {
        rules.push(json!({
            "id": issue.detector_name,
            "name": issue.title,
            "shortDescription": { "text": issue.title },
            "fullDescription": { "text": issue.description },
            "defaultConfiguration": { "level": level_for(issue.severity) },
        }));

        for inst in &issue.instances {
            results.push(json!({
                "ruleId": issue.detector_name,
                "level": level_for(issue.severity),
                "message": { "text": issue.description },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": inst.contract_path },
                        "region": {
                            "startLine": inst.line_no,
                            "snippet": { "text": if opts.no_snippets { "" } else { &inst.snippet } },
                        }
                    }
                }],
            }));
        }
    }

    (rules, results)
}

pub fn to_value(report: &Report, opts: RenderOptions) -> Value {
    let (rules, results) = rules_and_results(report, opts);
    json!({
        "$schema": SARIF_SCHEMA,
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "lancer-aderyn",
                    "informationUri": "https://github.com/NHTanvir/lancer-aderyn",
                    "version": report.tool_version,
                    "rules": rules,
                }
            },
            "results": results,
        }],
    })
}

pub fn render(report: &Report, opts: RenderOptions) -> String {
    serde_json::to_string_pretty(&to_value(report, opts)).unwrap_or_else(|_| "{}".to_string())
}
