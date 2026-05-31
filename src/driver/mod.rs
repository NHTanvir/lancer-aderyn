//! Orchestration: project discovery → solc → context build → detector run →
//! ignore-suppression → report assembly.

pub mod compile;
pub mod discovery;
pub mod ignore;

use std::collections::BTreeMap;
use std::path::PathBuf;

use eyre::Result;
use rayon::prelude::*;

use crate::config::AderynConfig;
use crate::context::WorkspaceContext;
use crate::detect::{get_all_detectors, IssueSeverity};
use crate::report::{FileSummary, Issue, IssueInstance, Report};

use ignore::IgnoreIndex;

/// Fully-resolved run options (built by the CLI / GUI).
#[derive(Clone, Debug)]
pub struct RunConfig {
    pub root: PathBuf,
    pub src: Option<String>,
    pub path_includes: Vec<String>,
    pub path_excludes: Vec<String>,
    pub output: String,
    pub stdout: bool,
    pub highs_only: bool,
    pub no_snippets: bool,
    pub skip_cloc: bool,
    pub skip_update_check: bool,
    pub auditor_mode: bool,
    pub auto_include_libs: bool,
    pub config: AderynConfig,
    /// Optional pre-built AST json (bypasses solc); honored when set.
    pub from_ast: Option<PathBuf>,
}

impl RunConfig {
    pub fn new(root: PathBuf) -> Self {
        RunConfig {
            root,
            src: None,
            path_includes: Vec::new(),
            path_excludes: Vec::new(),
            output: "report.md".to_string(),
            stdout: false,
            highs_only: false,
            no_snippets: false,
            skip_cloc: false,
            skip_update_check: false,
            auditor_mode: false,
            auto_include_libs: false,
            config: AderynConfig::default(),
            from_ast: None,
        }
    }
}

/// Build the `WorkspaceContext` for a run (discovery + compile + ignore index).
pub fn build_context(cfg: &RunConfig) -> Result<(WorkspaceContext, IgnoreIndex)> {
    // Pre-built AST short-circuit.
    if let Some(ast_path) = &cfg.from_ast {
        let ctx = compile::ingest_ast_json_file(ast_path)?;
        let mut ignores = IgnoreIndex::default();
        for f in &ctx.files {
            ignores.add_file(f.path.clone(), &f.content);
        }
        return Ok((ctx, ignores));
    }

    let layout = match discovery::detect_kind(&cfg.root) {
        discovery::ProjectKind::Foundry => discovery::read_foundry_layout(&cfg.root),
        _ => discovery::ProjectLayout::default(),
    };

    let src_dir = cfg.src.clone().or_else(|| layout.src_dir.clone());
    let mut excludes = cfg.path_excludes.clone();
    excludes.extend(cfg.config.exclude.iter().cloned());
    let mut includes = cfg.path_includes.clone();
    includes.extend(cfg.config.include.iter().cloned());

    let files = discovery::collect_solidity_files(
        &cfg.root,
        src_dir.as_deref(),
        &includes,
        &excludes,
    );

    let mut remappings = layout.remappings.clone();
    remappings.extend(cfg.config.remappings.iter().cloned());

    let ctx = compile::compile_to_context(&cfg.root, &files, &remappings, &cfg.config.env)?;

    let mut ignores = IgnoreIndex::default();
    for f in &ctx.files {
        ignores.add_file(f.path.clone(), &f.content);
    }

    Ok((ctx, ignores))
}

/// Run the full analysis and assemble a [`Report`].
pub fn analyze(cfg: &RunConfig) -> Result<Report> {
    let (ctx, ignores) = build_context(cfg)?;
    Ok(assemble_report(cfg, &ctx, &ignores))
}

/// Run detectors against an already-built context and assemble a report.
pub fn assemble_report(cfg: &RunConfig, ctx: &WorkspaceContext, ignores: &IgnoreIndex) -> Report {
    if cfg.auditor_mode {
        return crate::audit::collect(ctx);
    }

    // Select + run detectors.
    let mut detectors = get_all_detectors();
    detectors.retain(|d| {
        cfg.config.detector_enabled(&d.name())
            && (!cfg.highs_only || d.severity() == IssueSeverity::High)
    });

    detectors.par_iter_mut().for_each(|d| {
        if let Err(e) = d.detect(ctx) {
            tracing::warn!("detector `{}` errored: {e}", d.name());
        }
    });

    let mut highs: Vec<Issue> = Vec::new();
    let mut lows: Vec<Issue> = Vec::new();
    let mut detectors_used: Vec<(String, usize)> = Vec::new();

    for d in &detectors {
        let name = d.name();
        let hints = d.hints();
        let mut instances: Vec<IssueInstance> = Vec::new();
        for ((file, line, snippet), node_id) in d.instances() {
            if ignores.is_suppressed(&file, line, &name) {
                continue;
            }
            if ignores.is_false_positive(&file, line, &name) {
                // Known false positive: excluded from the main report.
                continue;
            }
            let hint = hints.get(&(file.clone(), line, snippet.clone())).cloned();
            instances.push(IssueInstance {
                contract_path: file,
                line_no: line,
                src: String::new(),
                snippet,
                node_id,
                hint,
            });
        }
        if instances.is_empty() {
            continue;
        }
        detectors_used.push((name.clone(), instances.len()));
        let issue = Issue {
            title: d.title(),
            description: d.description(),
            detector_name: name,
            severity: d.severity(),
            instances,
        };
        match d.severity() {
            IssueSeverity::High => highs.push(issue),
            IssueSeverity::Low => lows.push(issue),
        }
    }

    highs.sort_by(|a, b| a.detector_name.cmp(&b.detector_name));
    lows.sort_by(|a, b| a.detector_name.cmp(&b.detector_name));
    detectors_used.sort();

    let files_summary = if cfg.skip_cloc {
        Vec::new()
    } else {
        file_summaries(ctx)
    };
    let total_sloc = files_summary.iter().map(|f| f.n_sloc).sum();

    Report {
        highs,
        lows,
        files_summary,
        total_sloc,
        detectors_used,
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Run the analysis and emit the report (to file or stdout).
pub fn run(cfg: &RunConfig) -> Result<Report> {
    let report = analyze(cfg)?;
    let format = crate::report::ReportFormat::from_path(&cfg.output);
    let opts = crate::report::RenderOptions {
        no_snippets: cfg.no_snippets,
        highs_only: cfg.highs_only,
    };
    let rendered = crate::report::render(&report, format, opts);

    if cfg.stdout {
        println!("{rendered}");
    } else {
        std::fs::write(&cfg.output, rendered)?;
        tracing::info!("report written to {}", cfg.output);
    }
    Ok(report)
}

/// Count non-blank, non-comment "source lines of code" per file.
fn file_summaries(ctx: &WorkspaceContext) -> Vec<FileSummary> {
    let mut summaries: Vec<FileSummary> = ctx
        .files
        .iter()
        .map(|f| FileSummary {
            file_path: f.path.clone(),
            n_sloc: count_sloc(&f.content),
        })
        .collect();
    summaries.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    summaries
}

fn count_sloc(content: &str) -> usize {
    let mut count = 0usize;
    let mut in_block_comment = false;
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if in_block_comment {
            if line.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if line.starts_with("//") {
            continue;
        }
        if line.starts_with("/*") {
            if !line.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }
        count += 1;
    }
    count
}

/// Convenience: the detector-name → instance-count map for a report.
pub fn detector_histogram(report: &Report) -> BTreeMap<String, usize> {
    report
        .detectors_used
        .iter()
        .cloned()
        .collect::<BTreeMap<_, _>>()
}
