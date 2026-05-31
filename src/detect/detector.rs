//! The detector trait, severity enum, the `capture!` macro, and the registry
//! plumbing (`define_detectors!`, `get_all_detectors`, `request_detector_by_name`).

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::ast::NodeID;
use crate::context::{CaptureKey, WorkspaceContext};

/// Two-bucket severity, matching upstream (no critical/medium gradient).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    /// Code-quality / minor risk.
    Low,
    /// Very likely real exploitable issue or correctness bug.
    High,
}

impl fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueSeverity::Low => write!(f, "Low"),
            IssueSeverity::High => write!(f, "High"),
        }
    }
}

/// Every detector implements this trait. `detect()` populates the detector's
/// internal `found_instances` (via the `capture!` macro) and returns whether
/// anything was found.
pub trait IssueDetector: Send + Sync {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>>;

    fn severity(&self) -> IssueSeverity;

    /// Human-facing one-line title.
    fn title(&self) -> String;

    /// Longer remediation-oriented description.
    fn description(&self) -> String;

    /// Stable kebab-case identifier (used in config, registry, ignore directives).
    fn name(&self) -> String;

    /// The captured instances: `(file, line, snippet) -> node id`.
    fn instances(&self) -> BTreeMap<CaptureKey, NodeID>;

    /// Optional per-instance hints (extra remediation context).
    fn hints(&self) -> BTreeMap<CaptureKey, String> {
        BTreeMap::new()
    }
}

/// Insert a finding for `node` into the detector's `found_instances`.
///
/// ```ignore
/// capture!(self, context, member_access);
/// capture!(self, context, member_access, "consider SafeERC20");
/// ```
#[macro_export]
macro_rules! capture {
    ($self:expr, $context:expr, $node:expr) => {{
        #[allow(unused_imports)]
        use $crate::context::Capturable as _;
        let __cap = $node;
        if let Some(__key) = $context.capture_key(__cap.capturable_src()) {
            $self.found_instances.insert(__key, __cap.capturable_id());
        }
    }};
    ($self:expr, $context:expr, $node:expr, $hint:expr) => {{
        #[allow(unused_imports)]
        use $crate::context::Capturable as _;
        let __cap = $node;
        if let Some(__key) = $context.capture_key(__cap.capturable_src()) {
            $self
                .found_instances
                .insert(__key.clone(), __cap.capturable_id());
            $self.hints.insert(__key, ($hint).to_string());
        }
    }};
}

/// Generate the global detector registry from a list of detector structs.
/// Each struct must implement `IssueDetector + Default`.
#[macro_export]
macro_rules! define_detectors {
    ($($det:ty),+ $(,)?) => {
        /// Construct one instance of every registered detector.
        pub fn get_all_detectors() -> Vec<Box<dyn $crate::detect::IssueDetector>> {
            vec![ $( Box::<$det>::default() as Box<dyn $crate::detect::IssueDetector> ),+ ]
        }

        /// Total number of registered (active) detectors.
        pub fn detector_count() -> usize {
            get_all_detectors().len()
        }
    };
}

/// Look up a fresh detector instance by its `name()`.
pub fn request_detector_by_name(name: &str) -> Option<Box<dyn IssueDetector>> {
    crate::detect::registry::get_all_detectors()
        .into_iter()
        .find(|d| d.name() == name)
}

/// All detector names (sorted) — used by `registry` subcommand + GUI.
pub fn all_detector_names() -> Vec<String> {
    let mut names: Vec<String> = crate::detect::registry::get_all_detectors()
        .iter()
        .map(|d| d.name())
        .collect();
    names.sort();
    names
}
