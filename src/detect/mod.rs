//! The detection engine: the detector trait, helper predicates, the registry,
//! and the high/low detector collections.

pub mod detector;
pub mod helpers;
pub mod high;
pub mod low;
pub mod registry;

pub use detector::{
    all_detector_names, request_detector_by_name, IssueDetector, IssueSeverity,
};
pub use registry::{detector_count, get_all_detectors};
