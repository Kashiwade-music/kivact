//! Role: The meaning of a status, a notification or a message, as plain data.
//!
//! One vocabulary for "what is this about", shared by every standard
//! component and every kit, so that a call site reads the same whichever
//! theme draws it. A tone says nothing about color: the theme's recipes
//! decide how each one looks.

/// What a status, a notification or a message is about.
///
/// A [`Toast`](crate::Toast) carries one, a standard component or a kit
/// component takes one as a [`Variant`](https://docs.rs/kivact-ui) of its
/// recipes, and a recipe draws it in the theme's colors. [`Warning`](Self::Warning)
/// and [`Danger`](Self::Danger) are urgent: a toast of either is announced
/// assertively (`Role::Alert`, interrupting what is being read); the others
/// politely (`Role::Status`, after the current speech).
///
/// ```
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::Tone;
/// assert_eq!(Tone::default(), Tone::Neutral);
/// assert!(Tone::Danger.is_urgent());
/// assert!(!Tone::Success.is_urgent());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Tone {
    /// No particular meaning: plain text and no icon.
    #[default]
    Neutral,
    /// Information to note.
    Info,
    /// Something finished well.
    Success,
    /// Something needs attention. Urgent.
    Warning,
    /// Something failed or cannot be undone. Urgent.
    Danger,
    /// The theme's feature color, for emphasis that is not a status.
    Accent,
}

kivact_core::view::constant_value!(Tone);

impl Tone {
    /// Whether the tone asks for the user's attention at once
    /// ([`Warning`](Self::Warning) and [`Danger`](Self::Danger)): what makes
    /// a toast an alert rather than a status.
    pub fn is_urgent(self) -> bool {
        matches!(self, Self::Warning | Self::Danger)
    }
}
