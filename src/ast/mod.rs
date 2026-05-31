//! Solidity AST model.
//!
//! These types mirror the compact-JSON AST that `solc` emits (the same shape
//! consumed by Cyfrin's `solidity-ast` crate upstream). Every node carries an
//! `id` (`NodeID`, an `i64`) and a `src` location string of the form
//! `"<byteOffset>:<byteLength>:<sourceIndex>"`.
//!
//! We model this ourselves (rather than depending on an external AST crate) so
//! the detector API is fully under our control and internally consistent.

mod nodes;
pub use nodes::*;

/// solc node id. Unique within a compilation unit.
pub type NodeID = i64;

/// Parsed `src` location: `byteOffset:byteLength:sourceIndex`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SrcLocation {
    pub byte_offset: usize,
    pub byte_length: usize,
    pub source_index: i64,
}

impl SrcLocation {
    /// Parse a solc `src` string. Returns `None` on a malformed value.
    pub fn parse(src: &str) -> Option<SrcLocation> {
        let mut parts = src.split(':');
        let byte_offset = parts.next()?.parse().ok()?;
        let byte_length = parts.next()?.parse().ok()?;
        let source_index = parts.next().unwrap_or("0").parse().unwrap_or(0);
        Some(SrcLocation { byte_offset, byte_length, source_index })
    }
}

/// Compute the 1-based line number for a byte offset in `content`.
pub fn line_for_offset(content: &str, byte_offset: usize) -> usize {
    let clamped = byte_offset.min(content.len());
    // Count newlines up to the offset; +1 for a 1-based line number.
    content.as_bytes()[..clamped].iter().filter(|&&b| b == b'\n').count() + 1
}

/// Extract the source slice referenced by a `src` location.
pub fn slice_for_src<'a>(content: &'a str, src: &str) -> Option<&'a str> {
    let loc = SrcLocation::parse(src)?;
    let start = loc.byte_offset.min(content.len());
    let end = (loc.byte_offset + loc.byte_length).min(content.len());
    content.get(start..end)
}
