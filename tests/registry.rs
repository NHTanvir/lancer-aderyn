//! Registry / detector smoke tests (no solc required).

use lancer_aderyn::detect::{detector_count, get_all_detectors};

#[test]
fn expected_active_detector_count() {
    // 39 active high + 49 active low (the two `_template` scaffolds excluded).
    assert_eq!(detector_count(), 88, "active detector count drifted");
}

#[test]
fn detector_names_are_unique_and_kebab_case() {
    let detectors = get_all_detectors();
    let mut names: Vec<String> = detectors.iter().map(|d| d.name()).collect();
    let total = names.len();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), total, "duplicate detector name(s) detected");

    for name in &names {
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "detector name `{name}` is not kebab-case"
        );
        assert!(!name.is_empty(), "empty detector name");
    }
}

#[test]
fn detectors_have_titles_and_descriptions() {
    for d in get_all_detectors() {
        assert!(!d.title().trim().is_empty(), "{} has empty title", d.name());
        assert!(
            d.description().trim().len() > 10,
            "{} has too-short description",
            d.name()
        );
    }
}
