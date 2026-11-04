//! Role: Passive theme data for toggle button.
/// Opt-in text measurement for a toggle_button whose label changes weight when pressed.
/// Configure it through [`crate::UiTokens`] and use the same size as the toggle_button recipe.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ToggleLabelReservation {
    /// Let the visible label determine its own width.
    #[default]
    Natural,
    /// Reserve space for the same label shaped at another weight.
    Reserve {
        /// The weight used in the pressed appearance.
        weight: u16,
        /// The recipe's font size in logical pixels.
        size: f64,
        /// The recipe's font size at [`crate::ControlSize::ExtraSmall`].
        extra_small_size: f64,
    },
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ToggleButtonTokens {
    pub label_reservation: ToggleLabelReservation,
}
