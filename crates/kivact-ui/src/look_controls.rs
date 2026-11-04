//! Role: Default recipes for the radio group, slider, toasts and splitter.
use super::values::{AxisState, CheckableState, ControlSize, ToastGroupState};
use kivact_core::custom::PartState;
use kivact_core::style::{
    Align, Color, ColorToken, Cursor, Direction, Justify, Position, Style, Theme, pct, style,
};
use kivact_core::view::Orientation;
use kivact_interaction::{ToastState, Tone};

/// The standard checkbox and switch draw their indicators in the root.
pub(super) fn empty_toggle_part(_: &Theme, _: &CheckableState) -> Style {
    style().size(0, 0)
}

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
    if state.checked.is_checked() {
        s.border(1, ColorToken::Accent).color(ColorToken::Accent)
    } else if state.invalid {
        s.border(1, ColorToken::Danger)
    } else {
        s.border(1, ColorToken::Border)
    }
}

pub(super) fn radio_mark(_: &Theme, state: &CheckableState) -> Style {
    if state.checked.is_checked() {
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

/// The thickness of a track and the side of a thumb.
fn slider_metrics(t: &Theme, size: ControlSize) -> (f64, f64) {
    let thumb = match size {
        ControlSize::ExtraSmall | ControlSize::Small => (t.text().size).round(),
        ControlSize::Medium | ControlSize::Large => (t.text().size + 4.0).round(),
    };
    ((thumb / 3.0).round().max(2.0), thumb)
}

/// The track: a thin rounded bar as tall as the thumb, so the thumb fits
/// it; 160 px long when vertical.
pub(super) fn slider_track(t: &Theme, state: &AxisState) -> Style {
    let (bar, thumb) = slider_metrics(t, state.size);
    let s = style().shrink(0).cursor(Cursor::Pointer);
    // The bar is the box's middle third; its painted fill keeps the box's
    // full thumb-sized extent for the pointer.
    let s = s.paint(move |d, size| {
        let r = bar / 2.0;
        let rect = if size.width >= size.height {
            kivact_core::geometry::Rect::new(0.0, (size.height - bar) / 2.0, size.width, bar)
        } else {
            kivact_core::geometry::Rect::new((size.width - bar) / 2.0, 0.0, bar, size.height)
        };
        d.rounded_rect(rect, r, ColorToken::SurfaceAlt);
    });
    match state.orientation {
        Orientation::Vertical => s.width(thumb).height(160),
        _ => s.height(thumb).min_width(80),
    }
}

/// The filled part of the track, in the accent.
pub(super) fn slider_range(t: &Theme, state: &AxisState) -> Style {
    let (bar, _) = slider_metrics(t, state.size);
    style().paint(move |d, size| {
        let rect = if size.width >= size.height && size.height > 0.0 {
            kivact_core::geometry::Rect::new(0.0, (size.height - bar) / 2.0, size.width, bar)
        } else {
            kivact_core::geometry::Rect::new((size.width - bar) / 2.0, 0.0, bar, size.height)
        };
        d.rounded_rect(rect, bar / 2.0, ColorToken::Accent);
    })
}

/// A round thumb on the surface, ringed in the accent.
pub(super) fn slider_thumb(t: &Theme, state: &AxisState) -> Style {
    let (_, thumb) = slider_metrics(t, state.size);
    style()
        .size(thumb, thumb)
        .radius(thumb / 2.0)
        .background(ColorToken::Surface)
        .border(2, ColorToken::Accent)
        .shadow(kivact_core::draw::Shadow::new(0, 1, 2, ColorToken::Shadow))
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

/// The grip on a split's handle: a short rounded bar across it.
pub(super) fn split_grip(_: &Theme, state: &AxisState) -> Style {
    let s = style()
        .shrink(0)
        .align_self(Align::Center)
        .radius(2)
        .background(ColorToken::Border);
    match state.orientation {
        Orientation::Vertical => s.size(24, 4),
        _ => s.size(4, 24),
    }
}

/// The handle: a hairline in a hit area wide enough to grab.
pub(super) fn splitter_handle(_: &Theme, state: &AxisState) -> Style {
    let s = style()
        .shrink(0)
        .direction(Direction::Row)
        .justify_content(Justify::Center)
        .paint(|d, size| {
            if size.width <= size.height {
                d.fill_rect(
                    kivact_core::geometry::Rect::new(
                        (size.width - 1.0) / 2.0,
                        0.0,
                        1.0,
                        size.height,
                    ),
                    ColorToken::Border,
                );
            } else {
                d.fill_rect(
                    kivact_core::geometry::Rect::new(
                        0.0,
                        (size.height - 1.0) / 2.0,
                        size.width,
                        1.0,
                    ),
                    ColorToken::Border,
                );
            }
        })
        .hover(style().background(ColorToken::Hover))
        .pressed(style().background(ColorToken::Pressed));
    match state.orientation {
        Orientation::Vertical => s.height(6).cursor(Cursor::NsResize),
        _ => s.width(6).cursor(Cursor::EwResize),
    }
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
