//! Unit tests for the source-level ignore directives.

use lancer_aderyn::driver::ignore::IgnoreTable;

#[test]
fn ignore_current_line_named() {
    let src = "uint256 x = 1; // aderyn-ignore(todo)\n";
    let t = IgnoreTable::parse(src);
    assert!(t.is_suppressed(1, "todo"));
    assert!(!t.is_suppressed(1, "weak-randomness"));
}

#[test]
fn ignore_next_line_named() {
    let src = "// aderyn-ignore-next-line(weak-randomness)\nblock.timestamp;\n";
    let t = IgnoreTable::parse(src);
    assert!(t.is_suppressed(2, "weak-randomness"));
    assert!(!t.is_suppressed(1, "weak-randomness"));
}

#[test]
fn bare_ignore_suppresses_all() {
    let src = "doThing(); // aderyn-ignore\n";
    let t = IgnoreTable::parse(src);
    assert!(t.is_suppressed(1, "anything"));
    assert!(t.is_suppressed(1, "another-detector"));
}

#[test]
fn false_positive_is_separate_bucket() {
    let src = "transfer(x); // aderyn-fp(unsafe-erc20-functions)\n";
    let t = IgnoreTable::parse(src);
    assert!(t.is_false_positive(1, "unsafe-erc20-functions"));
    // FP is NOT counted as a plain ignore.
    assert!(!t.is_suppressed(1, "unsafe-erc20-functions"));
}

#[test]
fn multiple_detectors_in_one_directive() {
    let src = "x = 1; // aderyn-ignore(todo, empty-block)\n";
    let t = IgnoreTable::parse(src);
    assert!(t.is_suppressed(1, "todo"));
    assert!(t.is_suppressed(1, "empty-block"));
    assert!(!t.is_suppressed(1, "weak-randomness"));
}
