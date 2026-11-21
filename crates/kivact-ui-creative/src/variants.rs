//! Role: The Creative kit's variant types for the standard components.

/// The Creative inline notice appearance of a standard `kivact::ui::alert`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoticeVariant;
impl kivact::ui::theme::Variant for NoticeVariant {}
