//! The report data model + format dispatch. The driver builds a [`Report`];
//! the formatters in `markdown` / `json` / `sarif` render it.

pub mod json;
pub mod markdown;
pub mod sarif;

use crate::ast::NodeID;
use crate::detect::IssueSeverity;

/// One captured location for an issue.
#[derive(Clone, Debug)]
pub struct IssueInstance {
    /// Report-facing file path.
    pub contract_path: String,
    /// 1-based line number.
    pub line_no: usize,
    /// solc `src` location string.
    pub src: String,
    /// The single-line source snippet.
    pub snippet: String,
    /// Node id of the captured node.
    pub node_id: NodeID,
    /// Optional remediation hint.
    pub hint: Option<String>,
}

/// One detector's findings.
#[derive(Clone, Debug)]
pub struct Issue {
    pub title: String,
    pub description: String,
    pub detector_name: String,
    pub severity: IssueSeverity,
    pub instances: Vec<IssueInstance>,
}

/// Per-file line-count summary.
#[derive(Clone, Debug, Default)]
pub struct FileSummary {
    pub file_path: String,
    pub n_sloc: usize,
}

/// The complete analysis result.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub highs: Vec<Issue>,
    pub lows: Vec<Issue>,
    pub files_summary: Vec<FileSummary>,
    pub total_sloc: usize,
    /// `(detector_name, instance_count)` for every detector that fired.
    pub detectors_used: Vec<(String, usize)>,
    pub tool_version: String,
}

impl Report {
    pub fn high_issue_count(&self) -> usize {
        self.highs.iter().map(|i| i.instances.len()).sum()
    }
    pub fn low_issue_count(&self) -> usize {
        self.lows.iter().map(|i| i.instances.len()).sum()
    }
    pub fn all_issues(&self) -> impl Iterator<Item = &Issue> {
        self.highs.iter().chain(self.lows.iter())
    }
}

/// Output formats supported by the CLI / GUI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportFormat {
    Markdown,
    Json,
    Sarif,
}

impl ReportFormat {
    /// Pick a format from an output file extension (`.md` / `.json` / `.sarif`).
    pub fn from_path(path: &str) -> ReportFormat {
        let lower = path.to_lowercase();
        if lower.ends_with(".json") {
            ReportFormat::Json
        } else if lower.ends_with(".sarif") {
            ReportFormat::Sarif
        } else {
            ReportFormat::Markdown
        }
    }
}

/// Options affecting rendering.
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderOptions {
    pub no_snippets: bool,
    pub highs_only: bool,
}

/// Render `report` to a string in the requested `format`.
pub fn render(report: &Report, format: ReportFormat, opts: RenderOptions) -> String {
    match format {
        ReportFormat::Markdown => markdown::render(report, opts),
        ReportFormat::Json => json::render(report, opts),
        ReportFormat::Sarif => sarif::render(report, opts),
    }
}
