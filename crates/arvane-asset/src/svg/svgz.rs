//! Role: SVGZ decompression through a reader bounded by the encoded size limit.

use std::io::Read;

/// Why SVGZ did not decompress.
pub(crate) enum DecompressError {
    /// More than the limit once decompressed.
    TooLarge,
    /// Not a valid gzip stream.
    Malformed(String),
}

/// The plain text of gzip-compressed `bytes`, reading at most `limit + 1`
/// bytes of output (more is `TooLarge`).
pub(crate) fn decompress(bytes: &[u8], limit: u64) -> Result<Vec<u8>, DecompressError> {
    let mut plain = Vec::new();
    flate2::read::GzDecoder::new(bytes)
        .take(limit.saturating_add(1))
        .read_to_end(&mut plain)
        .map_err(|e| DecompressError::Malformed(e.to_string()))?;
    if plain.len() as u64 > limit {
        return Err(DecompressError::TooLarge);
    }
    Ok(plain)
}
