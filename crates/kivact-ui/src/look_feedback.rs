//! Role: Supply token-based default feedback, card and input-group recipes.
use super::{
    AlertState, AlertVariant, CardState, EmptyStateState, InputGroupMemberState, InputGroupState,
    Look,
};
use kivact_core::style::{Align, ColorToken, Direction, Justify, Style, Theme, pct, style};
use kivact_interaction::Tone;

pub(super) fn empty_root(t: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .direction(Direction::Column)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(t.space().md)
        .padding(t.space().xl)
        .width(pct(100))
        .min_width(0)
}
pub(super) fn empty_title(t: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .font_size(t.text().size + 4.0)
        .weight(t.text().weight_strong)
        .color(ColorToken::Text)
}
pub(super) fn empty_description(t: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .font_size(t.text().size)
        .color(ColorToken::TextMuted)
}
pub(super) fn empty_icon(_: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}
pub(super) fn empty_actions(t: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .direction(Direction::Row)
        .gap(t.space().sm)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

pub(super) fn alert_root(t: &Theme, s: &Look<AlertState>) -> Style {
    let color = match s.tone {
        Tone::Neutral => ColorToken::Border,
        Tone::Info | Tone::Accent => ColorToken::Accent,
        Tone::Success => ColorToken::Success,
        Tone::Warning => ColorToken::Warning,
        Tone::Danger => ColorToken::Danger,
    };
    let base = style()
        .direction(Direction::Column)
        .gap(t.space().sm)
        .padding(t.space().md)
        .radius(t.radius().md)
        .border(1, color)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
        .min_width(0);
    if s.variant::<AlertVariant>() == Some(&AlertVariant::Banner) {
        base.width(pct(100))
    } else {
        base
    }
}
pub(super) fn alert_title(t: &Theme, _: &Look<AlertState>) -> Style {
    style()
        .weight(t.text().weight_strong)
        .font_size(t.text().size)
}
pub(super) fn alert_description(t: &Theme, _: &Look<AlertState>) -> Style {
    style()
        .font_size(t.text().size)
        .color(ColorToken::TextMuted)
}
pub(super) fn alert_icon(_: &Theme, _: &Look<AlertState>) -> Style {
    style().shrink(0)
}
pub(super) fn alert_actions(t: &Theme, _: &Look<AlertState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().sm)
}
pub(super) fn alert_content(t: &Theme, _: &Look<AlertState>) -> Style {
    style().direction(Direction::Column).gap(t.space().sm)
}

pub(super) fn card_root(t: &Theme, s: &Look<CardState>) -> Style {
    let color = if s.selected == Some(true) {
        ColorToken::Accent
    } else {
        ColorToken::Border
    };
    let base = style()
        .direction(Direction::Column)
        .min_width(0)
        .radius(t.radius().md)
        .border(1, color)
        .background(ColorToken::Surface)
        .color(ColorToken::Text);
    if s.interactive {
        base.focus_visible(style().ring_with_offset(
            t.focus_ring(),
            t.focus_ring_offset(),
            ColorToken::Focus,
        ))
        .hover(style().background(ColorToken::SurfaceAlt))
        .pressed(style().background(ColorToken::Pressed))
        .disabled(style().opacity(t.disabled_opacity()))
    } else {
        base
    }
}
pub(super) fn card_region(t: &Theme, _: &Look<CardState>) -> Style {
    style()
        .direction(Direction::Column)
        .gap(t.space().sm)
        .padding(t.space().md)
        .min_width(0)
}
pub(super) fn card_media(_: &Theme, _: &Look<CardState>) -> Style {
    style().min_width(0)
}
pub(super) fn group_root(t: &Theme, _: &Look<InputGroupState>) -> Style {
    style()
        .direction(Direction::Column)
        .gap(t.space().sm)
        .min_width(0)
}
pub(super) fn group_label(t: &Theme, _: &Look<InputGroupState>) -> Style {
    style()
        .font_size(t.text().size)
        .weight(t.text().weight_strong)
}
pub(super) fn group_members(_: &Theme, s: &Look<InputGroupState>) -> Style {
    style()
        .direction(if s.direction == kivact_core::view::ReadingDirection::Rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .align_items(Align::Stretch)
        .min_width(0)
}
pub(super) fn group_addon(t: &Theme, s: &Look<InputGroupState>) -> Style {
    style()
        .height(super::look::height(t, s.size))
        .padding_x(t.space().sm)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border(1, ColorToken::Border)
        .background(ColorToken::SurfaceAlt)
        .color(ColorToken::Text)
        .font_size(t.text().size)
}

pub(super) fn group_member(t: &Theme, s: &Look<InputGroupMemberState>) -> Style {
    let rtl = s.group.direction == kivact_core::view::ReadingDirection::Rtl;
    let (left, right) = if rtl {
        (s.last, s.first)
    } else {
        (s.first, s.last)
    };
    let radius = t.radius().md;
    let base = style()
        .height(super::look::height(t, s.group.size))
        .radius_corners(
            if left { radius } else { 0.0 },
            if right { radius } else { 0.0 },
            if right { radius } else { 0.0 },
            if left { radius } else { 0.0 },
        );
    let base = if s.first {
        base
    } else if rtl {
        base.margin_right(-1)
    } else {
        base.margin_left(-1)
    };
    if s.focused { base.z_index(1) } else { base }
}
