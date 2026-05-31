//! Solidity → AST compilation. We mirror upstream: shell out to `solc` (the
//! standard-JSON interface) and deserialize the emitted AST into our typed
//! `WorkspaceContext`. A pre-built combined-AST JSON can also be ingested
//! directly (used by tests and the `--from-ast` escape hatch).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use eyre::{eyre, Context, Result};
use serde_json::{json, Value};

use crate::ast::SourceUnit;
use crate::context::{SourceFile, WorkspaceContext};

use super::discovery::relative_path;

/// Build a `WorkspaceContext` for the given source files via `solc`.
pub fn compile_to_context(
    root: &Path,
    files: &[PathBuf],
    remappings: &[String],
    env: &BTreeMap<String, String>,
) -> Result<WorkspaceContext> {
    if files.is_empty() {
        return Err(eyre!("no Solidity source files found to analyze"));
    }

    // Read every source file's content.
    let mut sources = serde_json::Map::new();
    let mut contents: BTreeMap<String, String> = BTreeMap::new();
    for file in files {
        let rel = relative_path(root, file);
        let content = std::fs::read_to_string(file)
            .with_context(|| format!("reading {}", file.display()))?;
        sources.insert(rel.clone(), json!({ "content": content }));
        contents.insert(rel, content);
    }

    let input = json!({
        "language": "Solidity",
        "sources": sources,
        "settings": {
            "remappings": remappings,
            "outputSelection": { "*": { "": ["ast"] } },
        }
    });

    let solc = std::env::var("SOLC_PATH").unwrap_or_else(|_| "solc".to_string());
    let mut cmd = Command::new(&solc);
    cmd.arg("--standard-json").current_dir(root);
    for (k, v) in env {
        cmd.env(k, v);
    }

    let output = run_with_stdin(cmd, input.to_string().as_bytes())
        .with_context(|| format!("invoking `{solc} --standard-json` (is solc installed?)"))?;

    let parsed: Value = serde_json::from_slice(&output)
        .context("parsing solc standard-json output")?;

    report_solc_errors(&parsed);

    ingest_standard_json(&parsed, &contents)
}

/// Ingest a pre-parsed solc standard-json `Value` into a context.
pub fn ingest_standard_json(
    parsed: &Value,
    contents: &BTreeMap<String, String>,
) -> Result<WorkspaceContext> {
    let mut ctx = WorkspaceContext::new();
    let Some(sources) = parsed.get("sources").and_then(|s| s.as_object()) else {
        return Err(eyre!("solc output contained no `sources` (compilation failed)"));
    };

    let mut ingested = 0usize;
    for (path, source) in sources {
        let Some(ast) = source.get("ast") else { continue };
        let source_index = source.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
        let unit: SourceUnit = match serde_json::from_value(ast.clone()) {
            Ok(u) => u,
            Err(e) => {
                tracing::warn!("skipping {path}: failed to deserialize AST: {e}");
                continue;
            }
        };
        let content = contents.get(path).cloned().unwrap_or_default();
        ctx.add_source_unit(
            SourceFile {
                path: path.clone(),
                absolute_path: path.clone(),
                content,
                source_index,
            },
            unit,
        );
        ingested += 1;
    }

    if ingested == 0 {
        return Err(eyre!("no source units could be parsed from solc output"));
    }

    ctx.build();
    Ok(ctx)
}

/// Ingest a standalone combined-AST JSON file (e.g. saved solc output).
pub fn ingest_ast_json_file(path: &Path) -> Result<WorkspaceContext> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading AST json {}", path.display()))?;
    let value: Value = serde_json::from_str(&text).context("parsing AST json")?;

    // Accept either standard-json (`{sources:{..}}`) or a single SourceUnit.
    if value.get("sources").is_some() {
        let contents = BTreeMap::new();
        return ingest_standard_json(&value, &contents);
    }

    let unit: SourceUnit = serde_json::from_value(value).context("expected a SourceUnit")?;
    let mut ctx = WorkspaceContext::new();
    ctx.add_source_unit(
        SourceFile {
            path: path.to_string_lossy().to_string(),
            absolute_path: path.to_string_lossy().to_string(),
            content: String::new(),
            source_index: 0,
        },
        unit,
    );
    ctx.build();
    Ok(ctx)
}

fn report_solc_errors(parsed: &Value) {
    if let Some(errors) = parsed.get("errors").and_then(|e| e.as_array()) {
        for err in errors {
            let severity = err.get("severity").and_then(|s| s.as_str()).unwrap_or("error");
            let message = err
                .get("formattedMessage")
                .or_else(|| err.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("<no message>");
            if severity == "error" {
                tracing::error!("solc: {message}");
            } else {
                tracing::debug!("solc {severity}: {message}");
            }
        }
    }
}

#[cfg(unix)]
fn run_with_stdin(mut cmd: Command, stdin_data: &[u8]) -> Result<Vec<u8>> {
    run_with_stdin_impl(&mut cmd, stdin_data)
}

#[cfg(not(unix))]
fn run_with_stdin(mut cmd: Command, stdin_data: &[u8]) -> Result<Vec<u8>> {
    run_with_stdin_impl(&mut cmd, stdin_data)
}

fn run_with_stdin_impl(cmd: &mut Command, stdin_data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(stdin_data)?;
    }
    let output = child.wait_with_output()?;
    Ok(output.stdout)
}
