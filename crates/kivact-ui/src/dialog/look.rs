//! Role: The dialog recipe state, tokens, recipe table and default looks.
use crate::look_common::mark;

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::style::{ColorToken, Position, Style, Theme, pct, style};
use kivact_interaction::DialogMode;
pub(crate) fn dialog_surface(t: &Theme, _: &DialogState) -> Style {
    style()
        .width(480)
        .max_width(pct(100))
        .background(ColorToken::Surface)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
}

pub(crate) fn dialog_title(t: &Theme, _: &DialogState) -> Style {
    style()
        .font_size(t.text().size + 4.0)
        .weight(t.text().weight_strong)
}

pub(crate) fn dialog_description(_: &Theme, _: &DialogState) -> Style {
    style().color(ColorToken::TextMuted)
}

/// The title over the description, clear of the close button.
pub(crate) fn dialog_header(t: &Theme, _: &DialogState) -> Style {
    style()
        .padding(t.space().lg)
        .padding_right(t.space().lg + 24.0 + t.space().sm)
        .gap(t.space().sm)
}

/// The media view's box, above the title.
pub(crate) fn dialog_media(_: &Theme, _: &DialogState) -> Style {
    style().shrink(0)
}

pub(crate) fn dialog_body(t: &Theme, _: &DialogState) -> Style {
    style().padding(t.space().lg)
}

pub(crate) fn dialog_footer(t: &Theme, _: &DialogState) -> Style {
    style().padding(t.space().lg).gap(t.space().sm)
}

pub(crate) fn dialog_scrim(_: &Theme, _: &DialogState) -> Style {
    style()
        .position(Position::Fixed)
        .inset(0)
        .background(kivact_core::style::Color::BLACK.with_alpha(0.35))
}

/// The close button at the surface's top right, around its `x` icon.
pub(crate) fn dialog_close(t: &Theme, _: &DialogState) -> Style {
    let inset = t.space().lg - 4.0;
    mark()
        .position(Position::Absolute)
        .top(inset)
        .right(inset)
        .size(24, 24)
        .radius(t.radius().sm)
        .color(ColorToken::TextMuted)
        .hover(style().background(ColorToken::Hover))
}

/// Pointer target for pulling a drawer toward its edge.
pub(crate) fn drawer_handle(_: &Theme, state: &DialogState) -> Style {
    if matches!(
        state.kind,
        kivact_interaction::DialogMode::Drawer(
            kivact_core::geometry::Edge::Left | kivact_core::geometry::Edge::Right
        )
    ) {
        style().width(24).height(48).shrink(0)
    } else {
        style().height(24).width(48).shrink(0)
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

/// Where a theme centers modal dialogs opened by an attached trigger.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogCenter {
    #[default]
    Anchor,
    Window,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DialogTokens {
    pub center: DialogCenter,
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
