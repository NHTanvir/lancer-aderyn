//! Entry point.
//!
//! Following the Lancer pattern, the plain `lancer-aderyn` invocation (no args)
//! opens the native GUI window. Any CLI arguments fall through to the full CLI
//! surface (the `--cli` flag forces this; `--gui` forces the window).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use lancer_aderyn::cli::{self, Cli};

fn main() -> ExitCode {
    init_tracing();

    let raw: Vec<String> = std::env::args().skip(1).collect();
    let force_gui = raw.iter().any(|a| a == "--gui");
    let force_cli = raw.iter().any(|a| a == "--cli");
    let filtered: Vec<String> = raw
        .into_iter()
        .filter(|a| a != "--cli" && a != "--gui")
        .collect();

    let want_gui = force_gui || (filtered.is_empty() && !force_cli);

    if want_gui {
        let initial_root = filtered.first().map(PathBuf::from);
        return match lancer_aderyn::gui::launch(initial_root) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("lancer-aderyn (gui): {e:?}");
                ExitCode::FAILURE
            }
        };
    }

    // CLI mode.
    let mut argv = vec![std::env::args().next().unwrap_or_else(|| "lancer-aderyn".into())];
    argv.extend(filtered);
    let cli = Cli::parse_from(argv);

    match cli::execute(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("lancer-aderyn: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    // Logs go to stderr so they never corrupt LSP/MCP stdio protocols.
    let filter = EnvFilter::try_from_env("ADERYN_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
