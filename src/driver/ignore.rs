//! Source-level ignore directives honored by the scanner:
//!
//! - `// aderyn-ignore-next-line(detector-name)` — suppress on the next line.
//! - `// aderyn-ignore(detector-name)` — suppress on the current line.
//! - `// aderyn-fp(detector-name)` — mark a known false positive (separate bucket).
//!
//! A bare directive with no `(name)` suppresses *all* detectors for that line.

use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::Regex;

static IGNORE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"aderyn-(ignore-next-line|ignore|fp)(?:\(([^)]*)\))?").unwrap()
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Directive {
    Ignore,
    IgnoreNextLine,
    FalsePositive,
}

/// Per-file suppression table: `line -> directive -> set of detector names`.
/// An empty name set means "suppress all detectors on this line".
#[derive(Default)]
pub struct IgnoreTable {
    /// `line_no -> (directive, names)`
    pub entries: HashMap<usize, Vec<(Directive, Vec<String>)>>,
}

impl IgnoreTable {
    /// Build the suppression table for a single file's source text.
    pub fn parse(content: &str) -> IgnoreTable {
        let mut table = IgnoreTable::default();
        for (idx, line) in content.lines().enumerate() {
            let line_no = idx + 1; // 1-based
            for cap in IGNORE_RE.captures_iter(line) {
                let kind = match &cap[1] {
                    "ignore-next-line" => Directive::IgnoreNextLine,
                    "fp" => Directive::FalsePositive,
                    _ => Directive::Ignore,
                };
                let names: Vec<String> = cap
                    .get(2)
                    .map(|m| {
                        m.as_str()
                            .split([',', ' '])
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();

                // Map directive to the affected line.
                let affected = match kind {
                    Directive::IgnoreNextLine => line_no + 1,
                    _ => line_no,
                };
                table.entries.entry(affected).or_default().push((kind, names));
            }
        }
        table
    }

    /// Is a finding for `detector_name` on `line_no` suppressed by ignore /
    /// ignore-next-line? (FalsePositive is reported separately, not here.)
    pub fn is_suppressed(&self, line_no: usize, detector_name: &str) -> bool {
        self.lookup(line_no, detector_name, |d| {
            matches!(d, Directive::Ignore | Directive::IgnoreNextLine)
        })
    }

    /// Is a finding flagged as a known false positive on `line_no`?
    pub fn is_false_positive(&self, line_no: usize, detector_name: &str) -> bool {
        self.lookup(line_no, detector_name, |d| d == Directive::FalsePositive)
    }

    fn lookup(&self, line_no: usize, detector_name: &str, pred: impl Fn(Directive) -> bool) -> bool {
        let Some(list) = self.entries.get(&line_no) else {
            return false;
        };
        list.iter().any(|(dir, names)| {
            pred(*dir) && (names.is_empty() || names.iter().any(|n| n == detector_name))
        })
    }
}

/// All files' ignore tables, keyed by report-facing path.
#[derive(Default)]
pub struct IgnoreIndex {
    pub by_file: HashMap<String, IgnoreTable>,
}

impl IgnoreIndex {
    pub fn add_file(&mut self, path: String, content: &str) {
        self.by_file.insert(path, IgnoreTable::parse(content));
    }

    pub fn is_suppressed(&self, file: &str, line_no: usize, detector_name: &str) -> bool {
        self.by_file
            .get(file)
            .map(|t| t.is_suppressed(line_no, detector_name))
            .unwrap_or(false)
    }

    pub fn is_false_positive(&self, file: &str, line_no: usize, detector_name: &str) -> bool {
        self.by_file
            .get(file)
            .map(|t| t.is_false_positive(line_no, detector_name))
            .unwrap_or(false)
    }
}
