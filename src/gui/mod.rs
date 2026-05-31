//! Native GUI (`lancer-aderyn` with no args) — a pure-Rust egui/eframe desktop
//! frontend over the same `core` library the CLI uses. Three-pane Lancer layout
//! plus a local HTTP bus for cross-tool integration.

use std::cell::Cell;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::detect::IssueSeverity;
use crate::driver::{analyze, RunConfig};
use crate::report::{render, RenderOptions, Report, ReportFormat};

/// Message from a background scan worker to the UI thread.
enum ScanMsg {
    Done(Report),
    Error(String),
}

/// A flattened finding row for the table.
#[derive(Clone)]
struct RowData {
    severity: String,
    detector: String,
    file: String,
    line: usize,
    title: String,
    description: String,
    snippet: String,
}

/// State shared with the local HTTP bus thread.
#[derive(Default)]
struct HttpShared {
    status: String,
    root: String,
    report: Option<Arc<Report>>,
}

pub struct AderynApp {
    root: String,
    highs_only: bool,
    filter: String,
    status: String,
    dark: bool,
    report: Option<Arc<Report>>,
    selected: Option<usize>,
    rx: Option<crossbeam_channel::Receiver<ScanMsg>>,
    http: Arc<Mutex<HttpShared>>,
    http_port: Option<u16>,
}

impl AderynApp {
    fn new(_cc: &eframe::CreationContext<'_>, initial_root: Option<PathBuf>) -> Self {
        let root = initial_root
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".to_string());
        let http = Arc::new(Mutex::new(HttpShared {
            status: "idle".to_string(),
            root: root.clone(),
            report: None,
        }));
        let http_port = start_http(http.clone());
        AderynApp {
            root,
            highs_only: false,
            filter: String::new(),
            status: "idle".to_string(),
            dark: true,
            report: None,
            selected: None,
            rx: None,
            http,
            http_port,
        }
    }

    fn counts(&self) -> (usize, usize) {
        self.report
            .as_ref()
            .map(|r| (r.high_issue_count(), r.low_issue_count()))
            .unwrap_or((0, 0))
    }

    fn build_rows(&self) -> Vec<RowData> {
        let mut rows = Vec::new();
        let Some(report) = &self.report else {
            return rows;
        };
        let filter = self.filter.to_lowercase();
        for issue in report.all_issues() {
            let sev = match issue.severity {
                IssueSeverity::High => "High",
                IssueSeverity::Low => "Low",
            };
            for inst in &issue.instances {
                if !filter.is_empty() {
                    let hay = format!(
                        "{} {} {}",
                        issue.detector_name, issue.title, inst.contract_path
                    )
                    .to_lowercase();
                    if !hay.contains(&filter) {
                        continue;
                    }
                }
                rows.push(RowData {
                    severity: sev.to_string(),
                    detector: issue.detector_name.clone(),
                    file: inst.contract_path.clone(),
                    line: inst.line_no,
                    title: issue.title.clone(),
                    description: issue.description.clone(),
                    snippet: inst.snippet.clone(),
                });
            }
        }
        rows
    }

    fn start_scan(&mut self, ctx: &egui::Context) {
        let (tx, rx) = crossbeam_channel::unbounded();
        let mut cfg = RunConfig::new(PathBuf::from(&self.root));
        cfg.highs_only = self.highs_only;
        self.status = "scanning…".to_string();
        if let Ok(mut h) = self.http.lock() {
            h.status = "scanning".to_string();
            h.root = self.root.clone();
        }
        std::thread::spawn(move || {
            let msg = match analyze(&cfg) {
                Ok(r) => ScanMsg::Done(r),
                Err(e) => ScanMsg::Error(e.to_string()),
            };
            let _ = tx.send(msg);
        });
        self.rx = Some(rx);
        ctx.request_repaint();
    }

    fn poll(&mut self, ctx: &egui::Context) {
        let mut clear = false;
        if let Some(rx) = &self.rx {
            match rx.try_recv() {
                Ok(ScanMsg::Done(report)) => {
                    let arc = Arc::new(report);
                    self.status = format!(
                        "done: {} high, {} low",
                        arc.high_issue_count(),
                        arc.low_issue_count()
                    );
                    if let Ok(mut h) = self.http.lock() {
                        h.report = Some(arc.clone());
                        h.status = self.status.clone();
                    }
                    self.report = Some(arc);
                    self.selected = None;
                    clear = true;
                }
                Ok(ScanMsg::Error(e)) => {
                    self.status = format!("error: {e}");
                    if let Ok(mut h) = self.http.lock() {
                        h.status = self.status.clone();
                    }
                    clear = true;
                }
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    ctx.request_repaint();
                }
                Err(_) => clear = true,
            }
        }
        if clear {
            self.rx = None;
        }
    }

    fn export(&self) {
        let Some(report) = &self.report else {
            return;
        };
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown", &["md"])
            .add_filter("JSON", &["json"])
            .add_filter("SARIF", &["sarif"])
            .set_file_name("report.md")
            .save_file()
        {
            let fmt = ReportFormat::from_path(&path.display().to_string());
            let out = render(report, fmt, RenderOptions::default());
            let _ = std::fs::write(&path, out);
        }
    }
}

impl eframe::App for AderynApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll(ctx);
        ctx.set_visuals(if self.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        let rows = self.build_rows();
        let selected = self.selected;
        let status = self.status.clone();
        let (high, low) = self.counts();
        let port = self.http_port;

        let do_run = Cell::new(false);
        let do_open = Cell::new(false);
        let do_export = Cell::new(false);
        let do_quit = Cell::new(false);
        let toggle_theme = Cell::new(false);
        let new_selected = Cell::new(selected);

        // ---- Top menu bar ------------------------------------------------
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Folder…").clicked() {
                        do_open.set(true);
                        ui.close_menu();
                    }
                    if ui.button("Export Report…").clicked() {
                        do_export.set(true);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        do_quit.set(true);
                        ui.close_menu();
                    }
                });
                ui.menu_button("Tools", |ui| {
                    if ui.button("Run Scan").clicked() {
                        do_run.set(true);
                        ui.close_menu();
                    }
                });
                ui.menu_button("Settings", |ui| {
                    if ui.button("Toggle Dark/Light").clicked() {
                        toggle_theme.set(true);
                        ui.close_menu();
                    }
                });
                ui.menu_button("Help", |ui| {
                    ui.label("lancer-aderyn");
                    ui.label(format!("v{}", crate::VERSION));
                    if let Some(p) = port {
                        ui.label(format!("bus: 127.0.0.1:{p}"));
                    }
                });
            });
        });

        // ---- Left: inputs / scope ---------------------------------------
        egui::SidePanel::left("inputs")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Scope");
                ui.add_space(6.0);
                ui.label("Project root / .sol file:");
                ui.add(egui::TextEdit::singleline(&mut self.root).desired_width(f32::INFINITY));
                if ui.button("Open…").clicked() {
                    do_open.set(true);
                }
                ui.add_space(8.0);
                ui.checkbox(&mut self.highs_only, "High-severity only");
                ui.add_space(8.0);
                ui.label("Filter:");
                ui.add(egui::TextEdit::singleline(&mut self.filter).desired_width(f32::INFINITY));
                ui.add_space(12.0);
                if ui
                    .add_sized([ui.available_width(), 32.0], egui::Button::new("▶  Run"))
                    .clicked()
                {
                    do_run.set(true);
                }
            });

        // ---- Right: selected finding detail -----------------------------
        if let Some(idx) = selected {
            if let Some(row) = rows.get(idx) {
                egui::SidePanel::right("detail")
                    .resizable(true)
                    .default_width(340.0)
                    .show(ctx, |ui| {
                        ui.heading(&row.title);
                        ui.add_space(4.0);
                        ui.label(format!("Detector: {}", row.detector));
                        ui.label(format!("Severity: {}", row.severity));
                        ui.label(format!("Location: {}:{}", row.file, row.line));
                        ui.separator();
                        ui.label(&row.description);
                        if !row.snippet.is_empty() {
                            ui.add_space(6.0);
                            ui.label("Snippet:");
                            ui.code(&row.snippet);
                        }
                    });
            }
        }

        // ---- Center: findings table -------------------------------------
        egui::CentralPanel::default().show(ctx, |ui| {
            use egui_extras::{Column, TableBuilder};
            ui.heading("Findings");
            ui.add_space(4.0);
            if rows.is_empty() {
                ui.label("No findings. Set a root and press Run.");
                return;
            }
            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::remainder())
                .column(Column::auto())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Sev");
                    });
                    header.col(|ui| {
                        ui.strong("Detector");
                    });
                    header.col(|ui| {
                        ui.strong("File");
                    });
                    header.col(|ui| {
                        ui.strong("Line");
                    });
                })
                .body(|mut body| {
                    for (i, row) in rows.iter().enumerate() {
                        let is_sel = selected == Some(i);
                        body.row(18.0, |mut r| {
                            r.col(|ui| {
                                if ui.selectable_label(is_sel, &row.severity).clicked() {
                                    new_selected.set(Some(i));
                                }
                            });
                            r.col(|ui| {
                                if ui.selectable_label(is_sel, &row.detector).clicked() {
                                    new_selected.set(Some(i));
                                }
                            });
                            r.col(|ui| {
                                if ui.selectable_label(is_sel, &row.file).clicked() {
                                    new_selected.set(Some(i));
                                }
                            });
                            r.col(|ui| {
                                ui.label(row.line.to_string());
                            });
                        });
                    }
                });
        });

        // ---- Bottom status bar ------------------------------------------
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Status: {status}"));
                ui.separator();
                ui.label(format!("Items: {}", rows.len()));
                ui.separator();
                ui.label(format!("{high} high, {low} low"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Export").clicked() {
                        do_export.set(true);
                    }
                    if ui.button("Run").clicked() {
                        do_run.set(true);
                    }
                    if ui.button("Open").clicked() {
                        do_open.set(true);
                    }
                });
            });
        });

        // ---- Apply deferred actions -------------------------------------
        self.selected = new_selected.get();
        if toggle_theme.get() {
            self.dark = !self.dark;
        }
        if do_open.get() {
            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                self.root = p.display().to_string();
            }
        }
        if do_run.get() {
            self.start_scan(ctx);
        }
        if do_export.get() {
            self.export();
        }
        if do_quit.get() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

/// Bind the local HTTP bus on an OS-allocated port, write the port to
/// `%TEMP%/lancer-aderyn.port`, and serve in a background thread. Returns the
/// chosen port (or `None` if binding failed — the GUI still runs).
fn start_http(shared: Arc<Mutex<HttpShared>>) -> Option<u16> {
    let server = tiny_http::Server::http(("127.0.0.1", 0)).ok()?;
    let port = server.server_addr().to_ip().map(|a| a.port())?;
    let _ = std::fs::write(
        std::env::temp_dir().join("lancer-aderyn.port"),
        port.to_string(),
    );

    std::thread::spawn(move || {
        for mut request in server.incoming_requests() {
            let method = request.method().as_str().to_string();
            let url = request.url().to_string();
            let (path, query) = url.split_once('?').unwrap_or((url.as_str(), ""));
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);

            let response_body = handle_http(&method, path, query, &body, &shared);
            let header =
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                    .expect("valid header");
            let response =
                tiny_http::Response::from_string(response_body).with_header(header);
            let _ = request.respond(response);
        }
    });
    Some(port)
}

fn query_param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k == key).then_some(v)
    })
}

fn handle_http(
    method: &str,
    path: &str,
    query: &str,
    body: &str,
    shared: &Arc<Mutex<HttpShared>>,
) -> String {
    match (method, path) {
        (_, "/v1/state") => {
            let guard = shared.lock().ok();
            let (status, high, low) = guard
                .as_ref()
                .map(|h| {
                    let (hi, lo) = h
                        .report
                        .as_ref()
                        .map(|r| (r.high_issue_count(), r.low_issue_count()))
                        .unwrap_or((0, 0));
                    (h.status.clone(), hi, lo)
                })
                .unwrap_or_else(|| ("unknown".to_string(), 0, 0));
            serde_json::json!({ "status": status, "high": high, "low": low }).to_string()
        }
        (_, "/v1/export") => {
            let fmt = match query_param(query, "format").unwrap_or("json") {
                "markdown" | "md" => ReportFormat::Markdown,
                "sarif" => ReportFormat::Sarif,
                _ => ReportFormat::Json,
            };
            let guard = shared.lock().ok();
            match guard.as_ref().and_then(|h| h.report.clone()) {
                Some(report) => render(&report, fmt, RenderOptions::default()),
                None => "{}".to_string(),
            }
        }
        ("POST", "/v1/import") => {
            // Accept a target path from the JSON body `{ "path": "..." }`.
            let path_value = serde_json::from_str::<serde_json::Value>(body)
                .ok()
                .and_then(|v| v.get("path").and_then(|p| p.as_str()).map(String::from));
            if let (Some(p), Ok(mut h)) = (path_value, shared.lock()) {
                h.root = p;
                serde_json::json!({ "ok": true, "root": h.root }).to_string()
            } else {
                serde_json::json!({ "ok": false }).to_string()
            }
        }
        _ => serde_json::json!({ "error": "not found" }).to_string(),
    }
}

/// Launch the native GUI window (blocking until the window closes).
pub fn launch(initial_root: Option<PathBuf>) -> eyre::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("lancer-aderyn"),
        ..Default::default()
    };
    eframe::run_native(
        "lancer-aderyn",
        options,
        Box::new(move |cc| Ok(Box::new(AderynApp::new(cc, initial_root)))),
    )
    .map_err(|e| eyre::eyre!("eframe failed: {e}"))
}
