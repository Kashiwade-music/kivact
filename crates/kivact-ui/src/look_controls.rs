//! Role: Default recipes for the radio group, slider, toasts and splitter.
use super::values::ToastGroupState;
use crate::control_size::ControlSize;
use crate::toggle::look::CheckableState;
use kivact_core::custom::PartState;
use kivact_core::style::{
    Align, Color, ColorToken, Direction, Justify, Position, Style, Theme, pct, style,
};
use kivact_core::view::Orientation;
use kivact_interaction::{ToastState, Tone};

/// The options in a column, or a row when horizontal.
pub(super) fn radio_group(t: &Theme, orientation: &Orientation) -> Style {
    let s = style().gap(t.space().md);
    match orientation {
        Orientation::Vertical => s.direction(Direction::Column),
        _ => s.direction(Direction::Row).wrap(true),
    }
}

/// An option: its indicator beside its label.
pub(super) fn radio_item(t: &Theme, state: &CheckableState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .radius(t.radius().sm)
        .font_size(match state.size {
            ControlSize::ExtraSmall | ControlSize::Small => t.text().size_small,
            ControlSize::Medium | ControlSize::Large => t.text().size,
        })
        .color(ColorToken::Text)
}

/// The circle of an option, the accent around the `dot` while chosen.
pub(super) fn radio_indicator(t: &Theme, state: &CheckableState) -> Style {
    let side = (t.text().size + 2.0).round();
    let s = style()
        .size(side, side)
        .shrink(0)
        .radius(side / 2.0)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(ColorToken::Surface);
    if state.is_checked() {
        s.border(1, ColorToken::Accent).color(ColorToken::Accent)
    } else if state.invalid {
        s.border(1, ColorToken::Danger)
    } else {
        s.border(1, ColorToken::Border)
    }
}

pub(super) fn radio_mark(_: &Theme, state: &CheckableState) -> Style {
    if state.is_checked() {
        style()
    } else {
        style().size(0, 0).opacity(0.0)
    }
}

pub(super) fn radio_description(t: &Theme, _: &CheckableState) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

/// The regions at the bottom right, above the page.
pub(super) fn toast_region(t: &Theme, _: &PartState) -> Style {
    style()
        .position(Position::Fixed)
        .right(t.space().xl)
        .bottom(t.space().xl)
        .width(320)
        .max_width(pct(100))
}

/// One live region's toasts in a column; a space above the assertive
/// group while both groups show toasts.
pub(super) fn toast_group(t: &Theme, state: &ToastGroupState) -> Style {
    let s = style().gap(t.space().sm);
    if state.count > 0 && state.above > 0 {
        s.margin_top(t.space().sm)
    } else {
        s
    }
}

/// A toast on the raised surface, its edge in the tone's color.
pub(super) fn toast(t: &Theme, state: &ToastState) -> Style {
    let edge: Color = match state.tone {
        Tone::Neutral | Tone::Info => ColorToken::Border.into(),
        Tone::Success => ColorToken::Success.into(),
        Tone::Warning => ColorToken::Warning.into(),
        Tone::Danger => ColorToken::Danger.into(),
        Tone::Accent => ColorToken::Accent.into(),
    };
    style()
        .direction(Direction::Row)
        .align_items(Align::Start)
        .gap(t.space().md)
        .padding(t.space().lg)
        .background(ColorToken::Raised)
        .border(1, ColorToken::Border)
        .border_left(3, edge)
        .radius(t.radius().md)
        .shadow(kivact_core::draw::Shadow::new(0, 4, 12, ColorToken::Shadow))
}

/// The tone's icon box: 16 px in the tone's color.
pub(super) fn toast_icon(_: &Theme, state: &ToastState) -> Style {
    let color: Color = match state.tone {
        Tone::Neutral => ColorToken::TextMuted.into(),
        Tone::Info | Tone::Accent => ColorToken::Accent.into(),
        Tone::Success => ColorToken::Success.into(),
        Tone::Warning => ColorToken::Warning.into(),
        Tone::Danger => ColorToken::Danger.into(),
    };
    style().size(16, 16).shrink(0).color(color)
}

pub(super) fn toast_title(t: &Theme, _: &ToastState) -> Style {
    style().weight(t.text().weight_strong)
}

pub(super) fn toast_body(t: &Theme, _: &ToastState) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

/// The close button, around its `x` icon.
pub(super) fn toast_close(t: &Theme, _: &ToastState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .shrink(0)
        .size(24, 24)
        .radius(t.radius().sm)
        .color(ColorToken::TextMuted)
        .hover(style().background(ColorToken::Hover))
}

/// Shared by all kits: the row owns size presence independently of card paint.
pub(super) fn toast_row(t: &Theme, _: &PartState) -> Style {
    let tokens = t.ext::<crate::UiTokens>();
    let motion = &tokens.toast.row_motion;
    let base = style()
        .shrink(0)
        .enter_size(kivact_core::view::Orientation::Vertical)
        .exit_size(kivact_core::view::Orientation::Vertical);
    if let Some(transition) = &motion.transition {
        base.padding_top(motion.gap)
            .transition(kivact_core::style::TransitionProp::Size, transition.clone())
    } else {
        base
    }
}
