//! Role: The kbd recipe state, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::style::{Align, Style, Theme, style};
pub(crate) fn kbd(_: &Theme, _: &Look<()>) -> Style {
    style().align_items(Align::Center).gap(4).flex_shrink(0)
}

pub(crate) fn kbd_key(t: &Theme, _: &Look<KbdKeyState>) -> Style {
    style()
        .min_height(20)
        .min_width(20)
        .padding_x(t.get(&kivact_core::style::space::SM_VAR))
        .border_radius(t.get(&kivact_core::style::radius::SM_VAR))
        .background(kivact_core::style::color::SURFACE)
        .border_bottom(2, kivact_core::style::color::BORDER)
        .color(kivact_core::style::color::TEXT)
        .font_size(12)
        .text_wrap(false)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KbdKeyState {
    pub glyph: String,
    pub label: String,
}

recipe_table! {
    /// The recipes of a keyboard shortcut: its root and its keys.
    KbdTheme {
        root(root_recipe): () = kbd,
        key(key_recipe): KbdKeyState = kbd_key,
    }
}
