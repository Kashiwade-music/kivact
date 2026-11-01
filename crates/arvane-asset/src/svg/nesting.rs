//! Role: Cheap pre-scan of SVG text for deep nesting and entity expansion.
//!
//! A cheap scan of SVG text before usvg parses it: the element nesting
//! depth and entities that would expand to markup. usvg converts the tree
//! recursively, so a deeply nested document overflows a thread's stack and
//! aborts the process (a panic cannot catch that); the scan rejects such a
//! document with a message first.
//!
//! Invariants:
//! - One pass over the bytes, no allocation, no recursion.
//! - It never counts fewer open elements than an XML parser would see:
//!   quoted attribute values may hold `>` and `/>`, comments, CDATA,
//!   processing instructions and the DOCTYPE (with its internal subset)
//!   open no element. An entity whose value holds markup (`<`) is refused,
//!   since its references could nest elements the scan does not see.
//! - Malformed text is not judged here: usvg reports it.

/// The deepest element nesting accepted (the root `<svg>` is depth 1).
pub(crate) const MAX_SVG_DEPTH: usize = 256;

/// Checks `xml` (plain SVG text) before parsing (see the module
/// invariants); the error is the reason in a message.
pub(crate) fn check(xml: &[u8]) -> Result<(), String> {
    let mut depth = 0_usize;
    let mut i = 0;
    while let Some(at) = find_byte(xml, i, b'<') {
        let rest = &xml[at..];
        i = if rest.starts_with(b"<!--") {
            after(xml, at + 4, b"-->")
        } else if rest.starts_with(b"<![CDATA[") {
            after(xml, at + 9, b"]]>")
        } else if rest.starts_with(b"<?") {
            after(xml, at + 2, b"?>")
        } else if rest.starts_with(b"<!") {
            doctype(xml, at + 2)?
        } else if rest.starts_with(b"</") {
            depth = depth.saturating_sub(1);
            find_byte(xml, at + 2, b'>').map_or(xml.len(), |end| end + 1)
        } else {
            let (end, self_closing) = start_tag(xml, at + 1);
            if !self_closing {
                depth += 1;
                if depth > MAX_SVG_DEPTH {
                    return Err(format!(
                        "elements are nested more than {MAX_SVG_DEPTH} levels deep"
                    ));
                }
            }
            end
        };
    }
    Ok(())
}

/// The index of the first `byte` at or after `from`.
fn find_byte(xml: &[u8], from: usize, byte: u8) -> Option<usize> {
    xml.get(from..)?
        .iter()
        .position(|b| *b == byte)
        .map(|p| from + p)
}

/// The index just past the first `end` at or after `from` (the end of the
/// text when there is none).
fn after(xml: &[u8], from: usize, end: &[u8]) -> usize {
    xml.get(from..)
        .and_then(|rest| rest.windows(end.len()).position(|w| w == end))
        .map_or(xml.len(), |p| from + p + end.len())
}

/// A start tag whose name begins at `from`: the index past its `>` and
/// whether it closes itself (`/>`). Quoted attribute values are skipped.
fn start_tag(xml: &[u8], from: usize) -> (usize, bool) {
    let mut i = from;
    let mut last = 0_u8;
    while let Some(&b) = xml.get(i) {
        match b {
            b'"' | b'\'' => {
                i = find_byte(xml, i + 1, b).map_or(xml.len(), |q| q + 1);
                last = b;
                continue;
            }
            b'>' => return (i + 1, last == b'/'),
            b if !b.is_ascii_whitespace() => last = b,
            _ => {}
        }
        i += 1;
    }
    (xml.len(), false)
}

/// A `<!…>` declaration (the DOCTYPE) whose body begins at `from`: the
/// index past its closing `>`, skipping quoted literals, the internal
/// subset and comments in it. An entity value holding `<` is refused.
fn doctype(xml: &[u8], from: usize) -> Result<usize, String> {
    let mut i = from;
    let mut subset = 0_usize;
    while let Some(&b) = xml.get(i) {
        match b {
            b'"' | b'\'' => {
                let end = find_byte(xml, i + 1, b).unwrap_or(xml.len());
                if xml[i + 1..end].contains(&b'<') {
                    return Err("entities that contain markup are not supported".to_owned());
                }
                i = end + 1;
                continue;
            }
            b'<' if xml[i..].starts_with(b"<!--") => {
                i = after(xml, i + 4, b"-->");
                continue;
            }
            b'[' => subset += 1,
            b']' => subset = subset.saturating_sub(1),
            b'>' if subset == 0 => return Ok(i + 1),
            _ => {}
        }
        i += 1;
    }
    Ok(xml.len())
}
