//! Role: Number, XML text and PNG data-URI encoding for the SVG writer.

use crate::{Result, UiError};
use kivact_runtime::ErrorKind;
use std::panic::Location;

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
pub(super) fn png_data_uri(
    width: u32,
    height: u32,
    rgba: &[u8],
    at: &'static Location<'static>,
) -> Result<String> {
    let mut png = Vec::new();
    let encode = |png: &mut Vec<u8>| -> std::result::Result<(), png::EncodingError> {
        let mut encoder = png::Encoder::new(png, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(rgba)?;
        writer.finish()
    };
    encode(&mut png).map_err(|error| {
        UiError::new(
            ErrorKind::Message,
            format!("PNG encoding of an image failed: {error}"),
            Some(at),
        )
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
