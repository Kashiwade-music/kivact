//! Role: Font files and family names.
//!
//! Invariants:
//! - A font file is accepted only when skrifa reads it as TTF, OTF or TTC
//!   and every face in it has a family name. WOFF and WOFF2 are rejected
//!   with a message that says to convert them to TTF or OTF.
//! - A face's family is its typographic family name (name ID 16) when it
//!   has one, else its family name (ID 1), in English when present. The
//!   names are the ones written in the font, never the file name.

use std::sync::Arc;

use kivact_types::Asset;
use kivact_types::shaped::FontSource;
use skrifa::MetadataProvider;
use skrifa::raw::FileRef;
use skrifa::string::StringId;

use crate::error::{AssetError, AssetErrorKind};
use crate::read::{self, Encoded};

/// The bytes of a font file, as given (embedded bytes are not copied).
#[derive(Clone, Debug)]
pub enum FontBytes {
    /// Bytes embedded in the executable.
    Static(&'static [u8]),
    /// Bytes read from a file or given at run time.
    Shared(Arc<[u8]>),
}

impl FontBytes {
    /// The bytes.
    pub fn as_slice(&self) -> &[u8] {
        match self {
            FontBytes::Static(b) => b,
            FontBytes::Shared(b) => b,
        }
    }

    /// The bytes as a font source shared with them (never copied).
    pub fn to_source(&self) -> FontSource {
        match self {
            FontBytes::Static(b) => FontSource::from_static(b),
            FontBytes::Shared(b) => FontSource::from_shared(Arc::clone(b)),
        }
    }
}

/// A checked font file: its bytes and the families it contains.
#[derive(Clone, Debug)]
pub struct FontFile {
    bytes: FontBytes,
    families: Vec<String>,
}

impl FontFile {
    /// The bytes to register with the host's text system.
    pub fn bytes(&self) -> &FontBytes {
        &self.bytes
    }

    /// The family names in the file, in face order without duplicates.
    pub fn families(&self) -> &[String] {
        &self.families
    }
}

/// Reads `asset` and checks it is a TTF, OTF or TTC font file.
pub fn load_font(asset: &Asset) -> Result<FontFile, AssetError> {
    let encoded = read::read(asset)?;
    if encoded.len() as u64 > crate::limits::MAX_ENCODED_BYTES {
        return Err(read::too_large(asset));
    }
    let families = families(&encoded)
        .map_err(|reason| AssetError::new(AssetErrorKind::Font, asset, reason))?;
    let bytes = match encoded {
        Encoded::Static(b) => FontBytes::Static(b),
        Encoded::Shared(b) => FontBytes::Shared(b),
    };
    Ok(FontFile { bytes, families })
}

/// The family names of the faces in `bytes`.
fn families(bytes: &[u8]) -> Result<Vec<String>, String> {
    if bytes.starts_with(b"wOFF") || bytes.starts_with(b"wOF2") {
        return Err(
            "WOFF and WOFF2 fonts are not supported; convert the font to TTF or OTF".to_owned(),
        );
    }
    let signature = bytes.get(..4).unwrap_or_default();
    if ![&[0, 1, 0, 0][..], b"OTTO", b"true", b"ttcf"].contains(&signature) {
        return Err("not a font file (supported: TTF, OTF and TTC)".to_owned());
    }
    let file = FileRef::new(bytes).map_err(|e| format!("cannot read the font: {e}"))?;
    let mut names: Vec<String> = Vec::new();
    for font in file.fonts() {
        let font = font.map_err(|e| format!("cannot read a font in the file: {e}"))?;
        let name = [StringId::TYPOGRAPHIC_FAMILY_NAME, StringId::FAMILY_NAME]
            .into_iter()
            .find_map(|id| font.localized_strings(id).english_or_first())
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| "a font in the file has no family name".to_owned())?;
        if !names.contains(&name) {
            names.push(name);
        }
    }
    if names.is_empty() {
        return Err("the file contains no font".to_owned());
    }
    Ok(names)
}
