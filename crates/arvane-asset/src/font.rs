//! Role: Font files and family names.
//!
//! Invariants:
//! - A font file is accepted only when skrifa reads it as TTF, OTF or TTC
//!   and every face in it has a family name. WOFF and WOFF2 are rejected
//!   with a message that says to convert them to TTF or OTF.
//! - A face's family is its typographic family name (name ID 16) when it
//!   has one, else its family name (ID 1), in English when present. The
//!   names are the ones written in the font, never the file name.
//! - The generic families map to per-OS candidate lists in one table
//!   ([`GenericFamily::candidates`]); hosts never keep their own lists.
//! - Resolution never fails: a family that is not installed resolves to the
//!   system UI font and is flagged missing (hosts report it once).

use std::sync::Arc;

use kivact_types::Asset;
use kivact_types::render::FontSource;
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

/// An operating system, for the generic family table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Os {
    /// Windows.
    Windows,
    /// macOS.
    MacOs,
    /// Linux and the other Unix desktops.
    Linux,
}

impl Os {
    /// The operating system this process runs on.
    pub fn current() -> Os {
        match std::env::consts::OS {
            "windows" => Os::Windows,
            "macos" | "ios" => Os::MacOs,
            _ => Os::Linux,
        }
    }
}

/// A CSS generic family name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenericFamily {
    /// `system-ui`.
    SystemUi,
    /// `sans-serif`.
    SansSerif,
    /// `serif`.
    Serif,
    /// `monospace`.
    Monospace,
}

impl GenericFamily {
    /// The generic family `name` stands for (ASCII case-insensitive, as in
    /// CSS), if any.
    pub fn parse(name: &str) -> Option<GenericFamily> {
        [
            ("system-ui", GenericFamily::SystemUi),
            ("sans-serif", GenericFamily::SansSerif),
            ("serif", GenericFamily::Serif),
            ("monospace", GenericFamily::Monospace),
        ]
        .into_iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name.trim()))
        .map(|(_, g)| g)
    }

    /// The installed families to try for this generic family on `os`, in
    /// order. When none is installed, the system UI font is used. An empty
    /// list means the system UI font itself (it has no public family name
    /// on macOS, and Linux desktops differ).
    pub fn candidates(self, os: Os) -> &'static [&'static str] {
        use GenericFamily as G;
        match (os, self) {
            (Os::Windows, G::SystemUi) => &["Segoe UI"],
            (Os::Windows, G::SansSerif) => &["Segoe UI", "Arial"],
            (Os::Windows, G::Serif) => &["Cambria", "Times New Roman"],
            (Os::Windows, G::Monospace) => &["Cascadia Mono", "Consolas", "Courier New"],
            (Os::MacOs, G::SystemUi) => &[],
            (Os::MacOs, G::SansSerif) => &["Helvetica Neue", "Helvetica"],
            (Os::MacOs, G::Serif) => &["Times New Roman", "Times"],
            (Os::MacOs, G::Monospace) => &["SF Mono", "Menlo", "Monaco"],
            (Os::Linux, G::SystemUi) => &[],
            (Os::Linux, G::SansSerif) => &["DejaVu Sans", "Liberation Sans", "Noto Sans"],
            (Os::Linux, G::Serif) => &["DejaVu Serif", "Liberation Serif", "Noto Serif"],
            (Os::Linux, G::Monospace) => &["DejaVu Sans Mono", "Liberation Mono", "Noto Sans Mono"],
        }
    }
}

/// The font a host should use for a family name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FamilyChoice {
    /// This installed or registered family.
    Family(String),
    /// The host's system UI font.
    SystemUi,
}

/// A family selection and the diagnostic fact a host may report.
///
/// [`resolve_family`] supplies the platform table; the Arvane facade's
/// text engine combines it with its system and registered font database.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FamilyResolution {
    /// The font to use.
    pub choice: FamilyChoice,
    /// The name was a family (not a generic one) that is not installed; the
    /// host may report it once per family. Empty and generic names are
    /// never missing, even when they fall back to the system UI font.
    pub missing: bool,
}

/// Resolves a family name or generic family on `os`, asking the host
/// whether a family is installed (registered fonts included):
/// - a generic family becomes its first installed candidate, else the
///   system UI font (never missing);
/// - an installed family stays itself;
/// - an empty name is the system UI font;
/// - any other name falls back to the system UI font and is missing.
pub fn resolve_family(
    name: &str,
    os: Os,
    mut is_installed: impl FnMut(&str) -> bool,
) -> FamilyResolution {
    let found = |choice| FamilyResolution {
        choice,
        missing: false,
    };
    if let Some(generic) = GenericFamily::parse(name) {
        let first = generic
            .candidates(os)
            .iter()
            .find(|c| is_installed(c))
            .map(|c| FamilyChoice::Family((*c).to_owned()));
        return found(first.unwrap_or(FamilyChoice::SystemUi));
    }
    let name = name.trim();
    if name.is_empty() {
        return found(FamilyChoice::SystemUi);
    }
    if is_installed(name) {
        return found(FamilyChoice::Family(name.to_owned()));
    }
    FamilyResolution {
        choice: FamilyChoice::SystemUi,
        missing: true,
    }
}
