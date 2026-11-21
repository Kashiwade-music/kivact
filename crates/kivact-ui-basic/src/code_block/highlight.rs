//! Role: The highlight callback of a Rust code block, from the central Rust tokens.
use kivact::style::StyledRange;
use kivact::ui::{SyntaxKind, syntax_ranges};
use std::ops::Range;

/// The colored ranges of `text`, the editor's line `index`, from the Rust
/// tokens of that line (see [`kivact::ui::rust_tokens`]); and the index of
/// the next line, the state the editor passes on.
pub fn rust_lines(
    tokens: &[Vec<(Range<usize>, SyntaxKind)>],
    text: &str,
    index: &usize,
) -> (Vec<StyledRange>, usize) {
    let line = tokens.get(*index).map_or(&[][..], Vec::as_slice);
    (syntax_ranges(text, line), index + 1)
}
