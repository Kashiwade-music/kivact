//! Role: Image format detection from the content, never from the file extension.
//!
//! Invariants:
//! - Only the signatures below are accepted; there is no fallback guess
//!   (treating anything unknown as SVG would make TIFF fail inside the SVG
//!   parser without saying why).
//! - An unsupported content gets a name when it is a known format, so the
//!   message says "TIFF" rather than "unknown".

/// A supported image format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    Png,
    Jpeg,
    Gif,
    WebP,
    Svg,
    /// Gzip-compressed SVG.
    Svgz,
}

impl Format {
    /// The format's name in messages.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Format::Png => "PNG",
            Format::Jpeg => "JPEG",
            Format::Gif => "GIF",
            Format::WebP => "WebP",
            Format::Svg => "SVG",
            Format::Svgz => "SVGZ",
        }
    }

    /// Whether it is a vector format (SVG, SVGZ).
    pub(crate) fn is_vector(self) -> bool {
        matches!(self, Format::Svg | Format::Svgz)
    }
}

/// The supported formats, for messages.
pub(crate) const SUPPORTED: &str = "PNG, JPEG, GIF, WebP, SVG and SVGZ";

/// The format of `bytes`, or the name of the unsupported content.
pub(crate) fn detect(bytes: &[u8]) -> Result<Format, String> {
    let starts = |sig: &[u8]| bytes.starts_with(sig);
    if starts(b"\x89PNG\r\n\x1a\n") {
        return Ok(Format::Png);
    }
    if starts(&[0xff, 0xd8, 0xff]) {
        return Ok(Format::Jpeg);
    }
    if starts(b"GIF87a") || starts(b"GIF89a") {
        return Ok(Format::Gif);
    }
    if starts(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        return Ok(Format::WebP);
    }
    if starts(&[0x1f, 0x8b]) {
        return Ok(Format::Svgz);
    }
    if looks_like_svg(bytes) {
        return Ok(Format::Svg);
    }
    Err(unsupported_name(bytes))
}

/// XML text (after an optional UTF-8 byte order mark and white space) that
/// contains an `<svg` element.
fn looks_like_svg(bytes: &[u8]) -> bool {
    let text = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    let start = text.iter().position(|b| !b.is_ascii_whitespace());
    start.is_some_and(|i| text[i] == b'<') && text.windows(4).any(|w| w == b"<svg")
}

/// A name for content that is not a supported format.
fn unsupported_name(bytes: &[u8]) -> String {
    let starts = |sig: &[u8]| bytes.starts_with(sig);
    let brand = bytes.get(4..12);
    let known = if bytes.is_empty() {
        Some("empty content")
    } else if starts(b"BM") {
        Some("BMP")
    } else if starts(b"II*\0") || starts(b"MM\0*") {
        Some("TIFF")
    } else if starts(&[0, 0, 1, 0]) {
        Some("ICO")
    } else if starts(b"qoif") {
        Some("QOI")
    } else if starts(b"8BPS") {
        Some("PSD")
    } else if starts(b"%PDF") {
        Some("PDF")
    } else if brand == Some(b"ftypavif") || brand == Some(b"ftypavis") {
        Some("AVIF")
    } else if brand.is_some_and(|b| b.starts_with(b"ftyphei") || b.starts_with(b"ftypmif1")) {
        Some("HEIF")
    } else if starts(b"\x00\x00\x00\x0cjP  ") {
        Some("JPEG 2000")
    } else if starts(b"wOFF") || starts(b"wOF2") {
        Some("a WOFF font")
    } else if starts(&[0, 1, 0, 0]) || starts(b"OTTO") || starts(b"ttcf") {
        Some("a font file")
    } else {
        None
    };
    match known {
        Some(name) => name.to_owned(),
        None => {
            let head: Vec<String> = bytes.iter().take(8).map(|b| format!("{b:02x}")).collect();
            format!("unknown content starting with {}", head.join(" "))
        }
    }
}
