//! Role: The dialog recipe state, tokens, recipe table and default looks.
use crate::look_common::mark;

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::style::{Position, Style, Theme, pct, style};
use kivact_interaction::DialogMode;
pub(crate) fn dialog_surface(t: &Theme, _: &DialogState) -> Style {
    style()
        .width(480)
        .max_width(pct(100))
        .background(kivact_core::style::color::SURFACE)
        .border(1, kivact_core::style::color::BORDER)
        .border_radius(t.get(&kivact_core::style::radius::MD_VAR))
}

pub(crate) fn dialog_title(t: &Theme, _: &DialogState) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_VAR) + 4.0)
        .font_weight(t.get(&kivact_core::style::font::WEIGHT_STRONG_VAR))
}

pub(crate) fn dialog_description(_: &Theme, _: &DialogState) -> Style {
    style().color(kivact_core::style::color::TEXT_MUTED)
}

/// The title over the description, clear of the close button.
pub(crate) fn dialog_header(t: &Theme, _: &DialogState) -> Style {
    style()
        .padding(t.get(&kivact_core::style::space::LG_VAR))
        .padding_right(
            t.get(&kivact_core::style::space::LG_VAR)
                + 24.0
                + t.get(&kivact_core::style::space::SM_VAR),
        )
        .gap(t.get(&kivact_core::style::space::SM_VAR))
}

/// The media view's box, above the title.
pub(crate) fn dialog_media(_: &Theme, _: &DialogState) -> Style {
    style().flex_shrink(0)
}

pub(crate) fn dialog_body(t: &Theme, _: &DialogState) -> Style {
    style().padding(t.get(&kivact_core::style::space::LG_VAR))
}

pub(crate) fn dialog_footer(t: &Theme, _: &DialogState) -> Style {
    style()
        .padding(t.get(&kivact_core::style::space::LG_VAR))
        .gap(t.get(&kivact_core::style::space::SM_VAR))
}

pub(crate) fn dialog_scrim(_: &Theme, _: &DialogState) -> Style {
    style()
        .position(Position::Fixed)
        .inset(0)
        .background(kivact_core::style::Color::BLACK.with_alpha(0.35))
}

/// The close button at the surface's top right, around its `x` icon.
pub(crate) fn dialog_close(t: &Theme, _: &DialogState) -> Style {
    let inset = t.get(&kivact_core::style::space::LG_VAR) - 4.0;
    mark()
        .position(Position::Absolute)
        .top(inset)
        .right(inset)
        .size(24, 24)
        .border_radius(t.get(&kivact_core::style::radius::SM_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
        .hover(style().background(kivact_core::style::color::HOVER))
}

/// Pointer target for pulling a drawer toward its edge.
pub(crate) fn drawer_handle(_: &Theme, state: &DialogState) -> Style {
    if matches!(
        state.kind,
        kivact_interaction::DialogMode::Drawer(
            kivact_core::geometry::Edge::Left | kivact_core::geometry::Edge::Right
        )
    ) {
        style().width(24).height(48).flex_shrink(0)
    } else {
        style().height(24).width(48).flex_shrink(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DialogState {
    pub input: PartState,
    pub open: bool,
    pub kind: DialogMode,
    /// The dialog has a [`media`](crate::Dialog::media) view in its header,
    /// which the header recipe lays out beside or above the title.
    pub media: bool,
}

recipe_table! {
    /// The recipes of a [`dialog`](super::dialog).
    DialogRecipes {
        surface(surface_recipe): DialogState = dialog_surface,
        title(title_recipe): DialogState = dialog_title,
        description(description_recipe): DialogState = dialog_description,
        header(header_recipe): DialogState = dialog_header,
        /// The box around the header's [`media`](crate::Dialog::media) view.
        media(media_recipe): DialogState = dialog_media,
        body(body_recipe): DialogState = dialog_body,
        footer(footer_recipe): DialogState = dialog_footer,
        scrim(scrim_recipe): DialogState = dialog_scrim,
        /// The header's close button, holding the theme's `x` icon.
        close(close_recipe): DialogState = dialog_close,
        drag_handle(drag_handle_recipe): DialogState = drawer_handle,
    }
}
