//! lancer-aderyn — a Rust reimplementation of Cyfrin's Aderyn Solidity static
//! analyzer, with a native GUI on top. The `core` (everything except `gui`,
//! `cli`, `lsp`, `mcp`) is frontend-agnostic and shared by every entrypoint.

pub mod ast;
pub mod audit;
pub mod cli;
pub mod config;
pub mod context;
pub mod detect;
pub mod driver;
pub mod gui;
pub mod lsp;
pub mod mcp;
pub mod report;
pub mod self_test;

/// The crate version string (from Cargo).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
