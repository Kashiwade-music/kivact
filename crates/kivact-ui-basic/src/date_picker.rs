//! Role: Basic visual treatment of standard date picker surfaces.
/// The edge and spacing of a standard date field popup.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DatePickerVariant {
    /// A calendar-sized surface without extra popup padding.
    #[default]
    Flush,
}
impl kivact::ui::Variant for DatePickerVariant {}
