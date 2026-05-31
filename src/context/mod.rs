//! The analysis context: the indexed workspace, navigation helpers, graphs,
//! and the `Capturable` trait that ties detectors to source locations.

pub mod browser;
pub mod capturable;
pub mod flow;
pub mod graph;
pub mod workspace_context;

pub use capturable::Capturable;
pub use graph::NodeGraph;
pub use workspace_context::{CaptureKey, SourceFile, WorkspaceContext};
