//! lancer-aderyn
//! Rust clone of aderyn. See plan.md (local, gitignored) for the spec.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "lancer-aderyn", version, about = "Rust clone of aderyn")]
struct Cli {}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let _cli = Cli::parse();
    println!("lancer-aderyn - scaffold ready. Implement per plan.md.");
    Ok(())
}
