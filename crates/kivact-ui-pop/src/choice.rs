//! Role: The segmented look of a radio group and the check row style.

use kivact::style::{Align, ColorToken, Direction, Style, Theme, style};
#[path = "choice_style.rs"]
pub(crate) mod appearance;

/// The row of a check: the drawn part beside the label. A disabled row
/// takes the theme's `disabled_opacity`, as every focus stop does.
pub(crate) fn check_style(_: &Theme) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(12)
        .min_height(22)
        .color(ColorToken::Text)
}

/// The pill look of a standard radio group, a segmented control:
/// `kivact::ui::radio_group(..).orientation(Orientation::Horizontal)
/// .variant(Segmented)`. Another theme draws the
/// plain radio group.
///
/// A radio group chooses exactly one segment: the arrow keys move and choose
/// (wrapping at the ends), a disabled segment is skipped and the chosen key
/// survives missing rows. Tabs that show content under them are
/// [`kivact::ui::tabs()`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Segmented;

impl kivact::ui::Variant for Segmented {}
