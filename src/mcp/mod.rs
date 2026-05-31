//! MCP server (`--mcp` / `mcp <transport>`) — exposes detectors as Model
//! Context Protocol tools so Claude / Cursor can call lancer-aderyn. We
//! implement a minimal JSON-RPC 2.0 server over stdio and an HTTP-stream
//! transport, with no external MCP crate.

use std::io::{BufRead, Read, Write};
use std::path::PathBuf;

use serde_json::{json, Value};

use crate::detect::{get_all_detectors, request_detector_by_name};
use crate::driver::{analyze, build_context, RunConfig};
use crate::report::RenderOptions;

fn ok_response(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn err_response(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn tools_list() -> Value {
    json!([
        {
            "name": "list_detectors",
            "description": "List every available lancer-aderyn detector with its severity and title.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "scan_path",
            "description": "Run all detectors on a Solidity project or file and return the JSON report.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Project root or .sol file" },
                    "highs_only": { "type": "boolean" }
                },
                "required": ["path"]
            }
        },
        {
            "name": "run_detector",
            "description": "Run a single named detector on a path and return its instances.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Detector name, e.g. weak-randomness" },
                    "path": { "type": "string" }
                },
                "required": ["name", "path"]
            }
        }
    ])
}

fn call_tool(name: &str, args: &Value) -> Result<String, String> {
    match name {
        "list_detectors" => {
            let list: Vec<Value> = get_all_detectors()
                .iter()
                .map(|d| {
                    json!({ "name": d.name(), "severity": d.severity().to_string(), "title": d.title() })
                })
                .collect();
            Ok(serde_json::to_string_pretty(&json!(list)).unwrap_or_default())
        }
        "scan_path" => {
            let path = args.get("path").and_then(|p| p.as_str()).ok_or("missing 'path'")?;
            let mut cfg = RunConfig::new(PathBuf::from(path));
            cfg.highs_only = args.get("highs_only").and_then(|b| b.as_bool()).unwrap_or(false);
            let report = analyze(&cfg).map_err(|e| e.to_string())?;
            let value = crate::report::json::to_value(
                &report,
                RenderOptions { no_snippets: false, highs_only: cfg.highs_only },
            );
            Ok(serde_json::to_string_pretty(&value).unwrap_or_default())
        }
        "run_detector" => {
            let path = args.get("path").and_then(|p| p.as_str()).ok_or("missing 'path'")?;
            let det_name = args.get("name").and_then(|n| n.as_str()).ok_or("missing 'name'")?;
            let cfg = RunConfig::new(PathBuf::from(path));
            let (ctx, _ignores) = build_context(&cfg).map_err(|e| e.to_string())?;
            let mut det = request_detector_by_name(det_name)
                .ok_or_else(|| format!("unknown detector: {det_name}"))?;
            det.detect(&ctx).map_err(|e| e.to_string())?;
            let instances: Vec<Value> = det
                .instances()
                .into_iter()
                .map(|((file, line, snippet), _id)| {
                    json!({ "file": file, "line": line, "snippet": snippet })
                })
                .collect();
            Ok(serde_json::to_string_pretty(
                &json!({ "detector": det_name, "instances": instances }),
            )
            .unwrap_or_default())
        }
        _ => Err(format!("unknown tool: {name}")),
    }
}

/// Handle one JSON-RPC request. Returns `None` for notifications (no reply).
pub fn dispatch(request: &Value) -> Option<Value> {
    let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = request.get("id").cloned();

    match method {
        "initialize" => Some(ok_response(
            id?,
            json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": { "name": "lancer-aderyn", "version": crate::VERSION },
                "capabilities": { "tools": {} }
            }),
        )),
        "notifications/initialized" | "initialized" => None,
        "ping" => Some(ok_response(id?, json!({}))),
        "tools/list" => Some(ok_response(id?, json!({ "tools": tools_list() }))),
        "tools/call" => {
            let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            let (text, is_error) = match call_tool(tool_name, &args) {
                Ok(t) => (t, false),
                Err(e) => (e, true),
            };
            Some(ok_response(
                id?,
                json!({ "content": [{ "type": "text", "text": text }], "isError": is_error }),
            ))
        }
        _ => id.map(|i| err_response(i, -32601, "Method not found")),
    }
}

/// MCP server over stdio (newline-delimited JSON-RPC).
pub fn serve_stdio() -> eyre::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("mcp: invalid json-rpc line: {e}");
                continue;
            }
        };
        if let Some(response) = dispatch(&request) {
            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
        }
    }
    Ok(())
}

/// MCP server over an HTTP-stream transport (one JSON-RPC body per request).
pub fn serve_http(port: u16) -> eyre::Result<()> {
    let server = tiny_http::Server::http(("127.0.0.1", port))
        .map_err(|e| eyre::eyre!("failed to bind 127.0.0.1:{port}: {e}"))?;
    eprintln!("lancer-aderyn MCP http-stream listening on http://127.0.0.1:{port}");

    for mut request in server.incoming_requests() {
        let mut body = String::new();
        let _ = request.as_reader().read_to_string(&mut body);

        let response_json = match serde_json::from_str::<Value>(&body) {
            Ok(req) => dispatch(&req).unwrap_or_else(|| json!({ "jsonrpc": "2.0", "result": null })),
            Err(e) => json!({
                "jsonrpc": "2.0",
                "error": { "code": -32700, "message": format!("parse error: {e}") }
            }),
        };

        let data = serde_json::to_string(&response_json).unwrap_or_default();
        let header =
            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                .expect("valid header");
        let response = tiny_http::Response::from_string(data).with_header(header);
        let _ = request.respond(response);
    }
    Ok(())
}
