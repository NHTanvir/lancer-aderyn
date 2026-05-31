//! The full command-line surface (clap 4), matching upstream aderyn's verified
//! 0.6.x flags + subcommands. The plain GUI launch lives in `main.rs`; this
//! module owns the `--cli` surface.

use std::io;
use std::path::PathBuf;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use eyre::Result;

use crate::config::{scaffold_toml, AderynConfig};
use crate::driver::RunConfig;

#[derive(Parser, Debug)]
#[command(
    name = "lancer-aderyn",
    version,
    about = "Rust clone of aderyn — Solidity static analyzer (Lancer ecosystem)",
    long_about = None,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true,
)]
pub struct Cli {
    #[command(flatten)]
    pub scan: ScanArgs,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Args, Debug, Default)]
pub struct ScanArgs {
    /// Foundry/Hardhat project root, or a single `.sol` file.
    #[arg(default_value = ".")]
    pub root: String,

    /// Source contracts directory (auto-captured from foundry.toml).
    #[arg(short = 's', long = "src")]
    pub src: Option<String>,

    /// Comma-delimited path fragments to explicitly include.
    #[arg(short = 'i', long = "path-includes", value_delimiter = ',', alias = "include-paths")]
    pub path_includes: Vec<String>,

    /// Comma-delimited path fragments to exclude.
    #[arg(short = 'x', long = "path-excludes", value_delimiter = ',', alias = "exclude")]
    pub path_excludes: Vec<String>,

    /// Output file path; extension chooses format (.md / .json / .sarif).
    #[arg(short = 'o', long = "output", default_value = "report.md")]
    pub output: String,

    /// Start the LSP server on stdio.
    #[arg(short = 'l', long = "lsp")]
    pub lsp: bool,

    /// Start the MCP server on stdio (shorthand for `mcp stdio`).
    #[arg(long = "mcp")]
    pub mcp: bool,

    /// Omit code snippets from the report.
    #[arg(long = "no-snippets")]
    pub no_snippets: bool,

    /// Only run high-severity detectors.
    #[arg(long = "highs-only")]
    pub highs_only: bool,

    /// Print the report to stdout instead of writing the output file.
    #[arg(long = "stdout")]
    pub stdout: bool,

    /// Skip cloc / line-count analysis.
    #[arg(long = "skip-cloc")]
    pub skip_cloc: bool,

    /// Disable the version-check call to GitHub releases.
    #[arg(long = "skip-update-check")]
    pub skip_update_check: bool,

    /// Output manual-audit helpers only (suppresses normal detector output).
    #[arg(long = "auditor-mode", alias = "auditor")]
    pub auditor_mode: bool,

    /// Run the bundled parity self-test matrix and exit.
    #[arg(long = "self-test")]
    pub self_test: bool,

    /// Also analyze library sources (lib/, node_modules/).
    #[arg(long = "auto-include-libs")]
    pub auto_include_libs: bool,

    /// Path to an `aderyn.toml` config file (defaults to <root>/aderyn.toml).
    #[arg(long = "config")]
    pub config: Option<PathBuf>,

    /// Analyze a pre-built combined-AST JSON instead of invoking solc.
    #[arg(long = "from-ast")]
    pub from_ast: Option<PathBuf>,

    /// Force GUI mode (open the desktop window).
    #[arg(long = "gui", hide = true)]
    pub gui: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Write an `aderyn.toml` scaffold to the project root.
    Init {
        /// Target directory (defaults to ".").
        #[arg(default_value = ".")]
        path: String,
    },
    /// Browse detectors: `registry all` or `registry <detector-name>`.
    Registry {
        /// Detector name, or "all" (default) to list every detector.
        #[arg(default_value = "all")]
        detector: String,
    },
    /// Start the MCP server.
    Mcp {
        /// Transport: `stdio` or `http-stream`.
        #[arg(value_enum, default_value_t = McpTransport::Stdio)]
        transport: McpTransport,
        /// Port for `http-stream` transport.
        #[arg(long, default_value_t = 6277)]
        port: u16,
    },
    /// Open the Cyfrin docs (optionally with a question forwarded to search).
    Docs {
        /// Optional natural-language question.
        question: Vec<String>,
    },
    /// Emit shell completion scripts.
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum McpTransport {
    Stdio,
    #[value(name = "http-stream")]
    HttpStream,
}

/// Build a [`RunConfig`] from parsed scan args (loading + merging config).
pub fn build_run_config(scan: &ScanArgs) -> Result<RunConfig> {
    let root = PathBuf::from(&scan.root);
    let config = match &scan.config {
        Some(path) => AderynConfig::load_from_file(path)?,
        None => AderynConfig::load_from_root(&root)?,
    };

    let mut cfg = RunConfig::new(root);
    cfg.src = scan.src.clone().or_else(|| config.src.clone());
    cfg.path_includes = scan.path_includes.clone();
    cfg.path_excludes = scan.path_excludes.clone();
    cfg.output = scan.output.clone();
    cfg.stdout = scan.stdout;
    cfg.highs_only = scan.highs_only;
    cfg.no_snippets = scan.no_snippets;
    cfg.skip_cloc = scan.skip_cloc;
    cfg.skip_update_check = scan.skip_update_check;
    cfg.auditor_mode = scan.auditor_mode;
    cfg.auto_include_libs = scan.auto_include_libs;
    cfg.from_ast = scan.from_ast.clone();
    cfg.config = config;
    Ok(cfg)
}

/// Execute a parsed CLI invocation.
pub fn execute(cli: Cli) -> Result<()> {
    if let Some(command) = cli.command {
        return execute_command(command);
    }

    let scan = cli.scan;

    // Mode dispatch on the scan args.
    if scan.self_test {
        let ok = crate::self_test::run();
        return if ok {
            Ok(())
        } else {
            Err(eyre::eyre!("self-test failed"))
        };
    }
    if scan.lsp {
        return crate::lsp::serve_stdio();
    }
    if scan.mcp {
        return crate::mcp::serve_stdio();
    }

    let cfg = build_run_config(&scan)?;
    let report = crate::driver::run(&cfg)?;

    if scan.stdout {
        // Report already printed.
    } else {
        eprintln!(
            "lancer-aderyn: {} high, {} low issue(s) — report at {}",
            report.high_issue_count(),
            report.low_issue_count(),
            cfg.output
        );
    }
    Ok(())
}

fn execute_command(command: Command) -> Result<()> {
    match command {
        Command::Init { path } => {
            let target = PathBuf::from(&path).join("aderyn.toml");
            if target.exists() {
                eprintln!("aderyn.toml already exists at {}", target.display());
            } else {
                std::fs::write(&target, scaffold_toml())?;
                println!("Wrote {}", target.display());
            }
            Ok(())
        }
        Command::Registry { detector } => {
            print_registry(&detector);
            Ok(())
        }
        Command::Mcp { transport, port } => match transport {
            McpTransport::Stdio => crate::mcp::serve_stdio(),
            McpTransport::HttpStream => crate::mcp::serve_http(port),
        },
        Command::Docs { question } => {
            open_docs(&question.join(" "));
            Ok(())
        }
        Command::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, name, &mut io::stdout());
            Ok(())
        }
    }
}

fn print_registry(detector: &str) {
    use crate::detect::{get_all_detectors, request_detector_by_name};

    if detector == "all" {
        let detectors = get_all_detectors();
        println!("{} detectors registered:\n", detectors.len());
        let mut rows: Vec<(String, String, String)> = detectors
            .iter()
            .map(|d| (d.name(), d.severity().to_string(), d.title()))
            .collect();
        rows.sort();
        for (name, sev, title) in rows {
            println!("  [{sev:<4}] {name:<36} {title}");
        }
        return;
    }

    match request_detector_by_name(detector) {
        Some(d) => {
            println!("Name:        {}", d.name());
            println!("Severity:    {}", d.severity());
            println!("Title:       {}", d.title());
            println!("Description: {}", d.description());
        }
        None => eprintln!("Unknown detector: {detector} (try `registry all`)"),
    }
}

fn open_docs(question: &str) {
    let url = if question.is_empty() {
        "https://docs.cyfrin.io/aderyn".to_string()
    } else {
        format!(
            "https://docs.cyfrin.io/aderyn?q={}",
            urlencode(question)
        )
    };
    println!("Opening docs: {url}");
    let _ = open_in_browser(&url);
}

fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            b' ' => "+".to_string(),
            other => format!("%{other:02X}"),
        })
        .collect()
}

fn open_in_browser(url: &str) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    Ok(())
}
