//! Role: Passive theme data for avatar.

/// A theme's avatar defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AvatarTokens {
    /// Whether an avatar shows its name in a tooltip when the caller leaves it to the theme.
    pub name_tooltip: bool,
}
