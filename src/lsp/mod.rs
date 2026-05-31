//! LSP server (`--lsp`) — exposes detector findings as editor diagnostics via
//! `tower-lsp` over stdio.

use std::path::Path;

use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::driver::{analyze, RunConfig};

struct Backend {
    client: Client,
}

/// Analyze a single `.sol` file and turn findings into LSP diagnostics.
fn analyze_file(path: &Path) -> Vec<Diagnostic> {
    let mut cfg = RunConfig::new(path.to_path_buf());
    cfg.skip_cloc = true;
    let report = match analyze(&cfg) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("lsp analysis failed for {}: {e}", path.display());
            return Vec::new();
        }
    };

    let mut diagnostics = Vec::new();
    for issue in report.all_issues() {
        let severity = match issue.severity {
            crate::detect::IssueSeverity::High => DiagnosticSeverity::ERROR,
            crate::detect::IssueSeverity::Low => DiagnosticSeverity::WARNING,
        };
        for inst in &issue.instances {
            let line = inst.line_no.saturating_sub(1) as u32;
            diagnostics.push(Diagnostic {
                range: Range::new(Position::new(line, 0), Position::new(line, 200)),
                severity: Some(severity),
                source: Some("aderyn".to_string()),
                message: format!("[{}] {}: {}", issue.detector_name, issue.title, issue.description),
                ..Default::default()
            });
        }
    }
    diagnostics
}

impl Backend {
    async fn refresh(&self, uri: Url) {
        let Ok(path) = uri.to_file_path() else {
            return;
        };
        let diagnostics =
            tokio::task::spawn_blocking(move || analyze_file(&path)).await.unwrap_or_default();
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "lancer-aderyn".to_string(),
                version: Some(crate::VERSION.to_string()),
            }),
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "lancer-aderyn LSP ready")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.refresh(params.text_document.uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.refresh(params.text_document.uri).await;
    }

    async fn did_change(&self, _params: DidChangeTextDocumentParams) {
        // Re-analysis happens on save (the driver reads from disk via solc).
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }
}

/// Start the LSP server on stdio (blocking).
pub fn serve_stdio() -> eyre::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let (service, socket) = LspService::new(|client| Backend { client });
        Server::new(stdin, stdout, socket).serve(service).await;
    });
    Ok(())
}
