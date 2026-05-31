//! JSON report emitter.

use serde_json::{json, Value};

use super::{RenderOptions, Report};

pub fn to_value(report: &Report, opts: RenderOptions) -> Value {
    let issue_to_json = |issue: &super::Issue| {
        json!({
            "title": issue.title,
            "description": issue.description,
            "detector_name": issue.detector_name,
            "instances": issue.instances.iter().map(|i| {
                let mut obj = json!({
                    "contract_path": i.contract_path,
                    "line_no": i.line_no,
                    "src": i.src,
                });
                if !opts.no_snippets {
                    obj["src_char"] = json!(i.snippet);
                }
                if let Some(hint) = &i.hint {
                    obj["hint"] = json!(hint);
                }
                obj
            }).collect::<Vec<_>>(),
        })
    };

    let mut value = json!({
        "tool_version": report.tool_version,
        "files_summary": {
            "total_source_units": report.files_summary.len(),
            "total_sloc": report.total_sloc,
        },
        "files_details": report.files_summary.iter().map(|f| json!({
            "file_path": f.file_path,
            "n_sloc": f.n_sloc,
        })).collect::<Vec<_>>(),
        "issue_count": {
            "high": report.high_issue_count(),
            "low": report.low_issue_count(),
        },
        "high_issues": {
            "issues": report.highs.iter().map(issue_to_json).collect::<Vec<_>>(),
        },
        "detectors_used": report.detectors_used.iter().map(|(n, c)| json!({
            "name": n, "instances": c,
        })).collect::<Vec<_>>(),
    });

    if !opts.highs_only {
        value["low_issues"] = json!({
            "issues": report.lows.iter().map(issue_to_json).collect::<Vec<_>>(),
        });
    }

    value
}

pub fn render(report: &Report, opts: RenderOptions) -> String {
    serde_json::to_string_pretty(&to_value(report, opts)).unwrap_or_else(|_| "{}".to_string())
}
