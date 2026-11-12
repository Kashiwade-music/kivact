//! Role: Number, color, XML text and PNG encoding of the SVG document.

use crate::SvgError;
use kivact_types::Rgba;

/// A number with at most three decimals, trailing zeros dropped.
pub(super) fn number(value: f64) -> String {
    let text = format!("{value:.3}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-0" {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}

/// `stroke-opacity="…"` (or `fill-opacity`) for a color that is not opaque.
pub(super) fn opacity_attribute(kind: &str, color: Rgba) -> String {
    let alpha = (color.a.clamp(0.0, 1.0) * 255.0).round() / 255.0;
    if alpha == 1.0 {
        String::new()
    } else {
        format!(" {kind}-opacity=\"{}\"", number(alpha))
    }
}

/// `#rrggbb` of a color (its alpha is an opacity attribute).
pub(super) fn hex(color: Rgba) -> String {
    let channel = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!(
        "#{:02x}{:02x}{:02x}",
        channel(color.r),
        channel(color.g),
        channel(color.b)
    )
}

/// Escape `text` for an XML text node or a double-quoted attribute.
pub(super) fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

/// A `data:image/png;base64,…` URI of straight-alpha RGBA8 pixels.
pub(super) fn png_data_uri(width: u32, height: u32, rgba: &[u8]) -> Result<String, SvgError> {
    let mut png = Vec::new();
    let encode = |png: &mut Vec<u8>| -> Result<(), png::EncodingError> {
        let mut encoder = png::Encoder::new(png, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(rgba)?;
        writer.finish()
    };
    encode(&mut png).map_err(|error| {
        SvgError::Malformed(format!("PNG encoding of an image failed: {error}"))
    })?;
    Ok(format!("data:image/png;base64,{}", base64(&png)))
}

/// Standard base64 with padding.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    //! Role: Tests of number, color, escaping and the PNG data URI.

    use super::{base64, escape, hex, number, opacity_attribute, png_data_uri};
    use kivact_types::Rgba;

    fn color(r: f64, g: f64, b: f64, a: f64) -> Rgba {
        Rgba { r, g, b, a }
    }

    #[test]
    fn numbers_keep_three_decimals_and_drop_trailing_zeros() {
        assert_eq!(number(10.0), "10");
        assert_eq!(number(2.5), "2.5");
        assert_eq!(number(0.1234), "0.123");
        assert_eq!(number(-0.0001), "0");
        assert_eq!(number(-2.0), "-2");
    }

    #[test]
    fn colors_are_hex_and_a_partial_alpha_is_an_opacity() {
        assert_eq!(hex(color(1.0, 0.5, 0.0, 1.0)), "#ff8000");
        assert_eq!(opacity_attribute("fill", color(0.0, 0.0, 0.0, 1.0)), "");
        assert_eq!(
            opacity_attribute("stroke", color(0.0, 0.0, 0.0, 0.5)),
            " stroke-opacity=\"0.502\""
        );
    }

    #[test]
    fn xml_special_characters_are_escaped() {
        assert_eq!(escape("a&b<c>\"d'e"), "a&amp;b&lt;c&gt;&quot;d&apos;e");
    }

    #[test]
    fn base64_pads_the_last_group() {
        assert_eq!(base64(b"Man"), "TWFu");
        assert_eq!(base64(b"Ma"), "TWE=");
        assert_eq!(base64(b"M"), "TQ==");
    }

    #[test]
    fn a_pixel_buffer_becomes_a_png_data_uri() {
        let uri = png_data_uri(1, 1, &[255, 0, 0, 255]).expect("one opaque pixel encodes");

        assert!(uri.starts_with("data:image/png;base64,iVBORw0KGgo"));
    }

    #[test]
    fn a_buffer_shorter_than_its_size_is_malformed() {
        let error = png_data_uri(2, 2, &[0, 0, 0, 0]).expect_err("the rows are missing");

        assert!(matches!(error, crate::SvgError::Malformed(_)));
    }
}
