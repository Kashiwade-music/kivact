//! Role: Which installed family a CSS family name draws with.

/// What a host draws a family name with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FamilyChoice {
    /// This installed family (or a CSS family list, as written).
    Family(String),
    /// The system UI font.
    SystemUi,
}

/// A family name resolved: what to draw with, and whether the name was
/// a family that is not installed (reported once by the host).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FamilyResolution {
    /// What to draw with.
    pub choice: FamilyChoice,
    /// The name was a specific family that is not installed.
    pub missing: bool,
}

/// A generic family name (CSS) Arvane resolves, ASCII case-insensitive.
pub(crate) fn generic(name: &str) -> Option<parley::GenericFamily> {
    let name = name.trim();
    [
        ("system-ui", parley::GenericFamily::SystemUi),
        ("sans-serif", parley::GenericFamily::SansSerif),
        ("serif", parley::GenericFamily::Serif),
        ("monospace", parley::GenericFamily::Monospace),
    ]
    .into_iter()
    .find(|(css, _)| css.eq_ignore_ascii_case(name))
    .map(|(_, generic)| generic)
}
