//! Role: The style helpers the Creative recipes share: control, field and tab looks.
//! Also the glass look, the tone colors and the splitter.
use crate::color::{
    ACCENT, ACCENT_HIGH, BUTTON_PRESSED, BUTTON_SURFACE, CANVAS, CONTROL_BORDER, CONTROL_SHADOW,
    FIELD, HAIR, MUTED, OVERLAY_BORDER, OVERLAY_SURFACE, SELECTION, STRONG, SUCCESS, TEXT,
    TEXT_HIGH, WARNING, WINDOW_SHADOW, X, Z,
};
use crate::size::{ROW, ROW_VAR};
use kivact::interaction::Tone;
use kivact::style::{Align, Color, FlexDirection, Style, Theme, style};

/// The thickness of a splitter between two panes, in logical pixels.
pub(crate) const SPLITTER_THICKNESS: f64 = 6.0;

/// The colors a Creative theme gives a semantic [`Tone`].
pub(crate) trait ToneExt {
    /// The tone's color.
    fn color(self) -> Color;
}

impl ToneExt for Tone {
    fn color(self) -> Color {
        match self {
            Self::Neutral => MUTED,
            Self::Info => Z,
            Self::Success => SUCCESS,
            Self::Warning => WARNING,
            Self::Danger => X,
            Self::Accent => ACCENT_HIGH,
        }
    }
}

/// The toolbar button look: a row of the control height.
pub(crate) fn button_style(t: &Theme) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(kivact::style::Justify::Center)
        .height(ROW)
        .gap(6)
        .padding_x(11)
        .border_radius(7)
        .background(BUTTON_SURFACE)
        .border(1, CONTROL_BORDER)
        .color(TEXT)
        .font_weight(500)
        .flex_shrink(0)
        .hover(style().background(OVERLAY_BORDER).color(TEXT_HIGH))
        .active(style().scale(0.97).background(BUTTON_PRESSED))
        .selected(
            style()
                .background(SELECTION)
                .color(ACCENT_HIGH)
                .border(1, t.get(&crate::color::ACCENT_VAR).with_alpha(0.35)),
        )
}

/// A check row: the indicator beside the label.
pub(crate) fn check_style() -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(7)
        .height(ROW)
        .border_radius(5)
        .color(TEXT)
}

/// A text field: a field-colored row of the control height.
pub(crate) fn field_style() -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(6)
        .height(ROW)
        .padding_x(9)
        .border_radius(7)
        .background(FIELD)
        .color(TEXT)
        .border(1, CONTROL_BORDER)
        .hover(style().background(kivact::style::color::HOVER))
}

/// The segmented control's track.
pub(crate) fn segmented_style() -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .gap(2)
        .padding(2)
        .border_radius(9)
        .background(CONTROL_SHADOW)
        .border(1, HAIR)
}

/// One segment of a segmented control.
pub(crate) fn segmented_option_style(t: &Theme) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .min_width(0)
        .text_wrap(false)
        .text_overflow(kivact::style::TextOverflow::Ellipsis)
        .gap(5)
        .height(t.get(&ROW_VAR) - 4.0)
        .padding_x(10)
        .border_radius(5)
        .align_items(Align::Center)
        .color(MUTED)
        .hover(
            style()
                .background(kivact::style::color::HOVER)
                .color(TEXT_HIGH),
        )
        .selected(style().background(STRONG).color(TEXT_HIGH))
}

/// The surface of a floating panel: a blurred glass card.
pub(crate) fn glass_style() -> Style {
    style()
        .padding(6)
        .gap(1)
        .min_width(220)
        .border_radius(12)
        .color(TEXT)
        .background(OVERLAY_SURFACE)
        .backdrop_blur(22)
        .border(1, OVERLAY_BORDER)
        .shadow(kivact::style::Shadow::new(0.0, 18.0, 45.0, WINDOW_SHADOW))
}

/// The track of a tab list.
pub(crate) fn tab_track_style() -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .gap(2)
        .padding(3)
        .border_radius(10)
        .background(CANVAS)
        .border(1, HAIR)
        .min_width(0)
}

/// One tab of a tab list; a reorderable tab grabs the pointer.
pub(crate) fn tab_option_style(t: &Theme, reorderable: bool) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .height(t.get(&ROW_VAR) + 2.0)
        .padding_x(12)
        .flex_shrink(0)
        .gap(8)
        .border_radius(7)
        .align_items(Align::Center)
        .justify_content(kivact::style::Justify::Center)
        .color(MUTED)
        .cursor(if reorderable {
            kivact::style::Cursor::Grab
        } else {
            kivact::style::Cursor::Default
        })
        .selected(style().background(STRONG).color(TEXT_HIGH))
        .hover(style().color(TEXT_HIGH))
}

/// How the dock draws a splitter: transparent, accent while hovered or
/// focused, above the panes it separates. The splitter recipe shares it.
pub(crate) fn splitter_look() -> Style {
    style()
        .z_index(1)
        .hover(style().background(ACCENT))
        .focus(style().background(ACCENT))
}
