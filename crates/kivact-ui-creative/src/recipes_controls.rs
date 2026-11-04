//! Role: Creative appearance of the standard radio group, slider, toasts and splitter.
use crate::Color;

use kivact::custom::PartState;
use kivact::draw::Shadow;
use kivact::geometry::{Rect, Size};
use kivact::interaction::DisclosureState;
use kivact::interaction::ToastState;
use kivact::style::{Align, Cursor, Direction, Justify, Position, Style, Theme, pct, rgba, style};
use kivact::ui::{AxisState, CheckableState, RadioGroupRecipes, SliderRecipes};
use kivact::ui::{
    DisclosureRecipes, Look, ProgressRecipes, ProgressState, SplitterRecipes, ToastRecipes,
};
use kivact::view::Orientation;

use crate::choice::Segmented;
use crate::tokens::Tokens;

pub(crate) fn radio_group(r: RadioGroupRecipes) -> RadioGroupRecipes {
    r.group(|_| radio_group_layout)
        .item(|_| radio_item)
        .indicator(|_| radio_indicator)
        .description(|_| {
            |_: &Theme, _: &Look<CheckableState>| style().font_size(10.5).color(Color::Muted)
        })
}
fn radio_group_layout(t: &Theme, orientation: &Look<Orientation>) -> Style {
    if orientation.variant::<Segmented>().is_some() {
        return crate::control_style::segmented_style(t);
    }
    let s = style().gap(8);
    match **orientation {
        Orientation::Vertical => s.direction(Direction::Column),
        _ => s.direction(Direction::Row).wrap(true).gap(12),
    }
}
/// A radio row: the indicator beside the label.
fn radio_item(t: &Theme, state: &Look<CheckableState>) -> Style {
    if state.variant::<Segmented>().is_some() {
        return crate::control_style::segmented_option_style(t);
    }
    crate::control_style::check_style(t)
}
/// The 14 px disc of an option: the accent around the theme's `dot`, in the
/// accent's ink, while chosen.
fn radio_indicator(t: &Theme, s: &Look<CheckableState>) -> Style {
    if s.variant::<Segmented>().is_some() {
        // A segment shows its choice by its fill, without a disc.
        return style().position(Position::Absolute).size(0, 0).clip(true);
    }
    let p = t.ext::<Tokens>();
    let on = s.checked.is_checked();
    style()
        .size(14, 14)
        .shrink(0)
        .radius(7)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(if on { p.accent } else { p.strong })
        .color(p.accent_ink)
}

/// The thickness of a slider's bar and the side of its thumb.
const BAR: f64 = 4.0;
const THUMB: f64 = 16.0;
pub(crate) fn slider(r: SliderRecipes) -> SliderRecipes {
    r.track(|_| slider_track)
        .range(|_| slider_range)
        .thumb(|_| slider_thumb)
}
/// The bar of a track or of its filled range, across the middle of `size`.
fn bar(size: Size) -> Rect {
    if size.width >= size.height {
        Rect::new(0.0, (size.height - BAR) / 2.0, size.width, BAR)
    } else {
        Rect::new((size.width - BAR) / 2.0, 0.0, BAR, size.height)
    }
}
fn slider_track(t: &Theme, s: &Look<AxisState>) -> Style {
    let color = t.ext::<Tokens>().strong;
    let base = style()
        .shrink(0)
        .cursor(Cursor::EwResize)
        .paint(move |d, size| d.rounded_rect(bar(size), BAR / 2.0, color));
    match s.orientation {
        Orientation::Vertical => base.width(THUMB).height(160).cursor(Cursor::NsResize),
        _ => base.height(24).min_width(80),
    }
}
fn slider_range(t: &Theme, _: &Look<AxisState>) -> Style {
    let color = t.ext::<Tokens>().accent;
    style().paint(move |d, size| {
        if size.width > 0.0 && size.height > 0.0 {
            d.rounded_rect(bar(size), BAR / 2.0, color);
        }
    })
}
/// A 16 px disc in the high text color, like the range's endpoints.
fn slider_thumb(t: &Theme, _: &Look<AxisState>) -> Style {
    style()
        .size(THUMB, THUMB)
        .radius(THUMB / 2.0)
        .background(t.ext::<Tokens>().text_high)
}

pub(crate) fn toast(r: ToastRecipes) -> ToastRecipes {
    r.region(|_| toast_region)
        .toast(|_| toast_card)
        .title(|_| |_: &Theme, _: &Look<ToastState>| style().weight(600))
        .body(|_| |_: &Theme, _: &Look<ToastState>| style().font_size(11.5))
        .icon(|_| |_: &Theme, _: &Look<ToastState>| style().size(16, 16).shrink(0))
        .close(|_| toast_close)
}
/// The stack at the bottom right of the window, over scrolling content.
fn toast_region(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .position(Position::Fixed)
        .z_index(100)
        .max_width(pct(100) - 16.0)
        .align_items(Align::End)
        .gap(8)
        .bottom(16)
        .right(16)
}
/// A glass card edged in the tone's color.
fn toast_card(t: &Theme, s: &Look<ToastState>) -> Style {
    let p = t.ext::<Tokens>();
    let color = s.tone.color(&p);
    style()
        .direction(Direction::Row)
        .width(340)
        .max_width(pct(100))
        .padding(12)
        .gap(10)
        .radius(9)
        .align_items(Align::Start)
        .color(p.text)
        .background(rgba(0x1e1f26, 0.94))
        .backdrop_blur(22)
        .border(1, color.with_alpha(0.22))
        .shadow(Shadow::new(0.0, 12.0, 30.0, rgba(0x000000, 0.6)))
}
/// The square close button around the theme's `x`, like a toolbar action.
fn toast_close(t: &Theme, _: &Look<ToastState>) -> Style {
    crate::controls::button_style(t)
        .width(t.ext::<Tokens>().row)
        .padding(0)
}

pub(crate) fn splitter(r: SplitterRecipes) -> SplitterRecipes {
    r.handle(|_| splitter_handle)
}
/// The dock's splitter: a transparent bar, accent while hovered or focused.
fn splitter_handle(t: &Theme, s: &Look<AxisState>) -> Style {
    let thickness = crate::dock_style::SPLITTER_THICKNESS;
    let look = crate::dock_style::splitter_look(t);
    match s.orientation {
        Orientation::Vertical => look
            .height(thickness)
            .width(pct(100))
            .shrink(0)
            .cursor(Cursor::NsResize),
        _ => look
            .width(thickness)
            .height(pct(100))
            .shrink(0)
            .cursor(Cursor::EwResize),
    }
}

/// A bar of the field color, filled with the accent from the shown fraction.
pub(crate) fn progress(r: ProgressRecipes) -> ProgressRecipes {
    r.track(|_| progress_track).fill(|_| progress_fill)
}

fn progress_track(_: &Theme, _: &Look<ProgressState>) -> Style {
    style()
        .height(8)
        .width(pct(100))
        .radius(4)
        .clip(true)
        .background(Color::Field)
}

fn progress_fill(_: &Theme, look: &Look<ProgressState>) -> Style {
    let state = **look;
    style()
        .width(pct(100))
        .height(pct(100))
        .paint(move |d, size| {
            let (x, width) = if state.value.is_some() {
                (0.0, size.width * state.shown.get().clamp(0.0, 1.0))
            } else {
                let (start, length) = state.sweep();
                (size.width * start, size.width * length)
            };
            d.rounded_rect(
                Rect::new(x, 0.0, width, size.height),
                4.0,
                d.theme().ext::<Tokens>().accent,
            );
        })
}

/// An inspector section: an island whose header row starts with the chevron.
pub(crate) fn disclosure(r: DisclosureRecipes) -> DisclosureRecipes {
    r.root(|_| disclosure_root)
        .trigger(|_| disclosure_trigger)
        .indicator(|_| disclosure_indicator)
        .content(|_| disclosure_content)
}

fn disclosure_root(_: &Theme, _: &Look<DisclosureState>) -> Style {
    style()
        .direction(Direction::Column)
        .radius(7)
        .background(Color::Island)
        .border(1, Color::Hair)
}

/// The chevron is the trigger's last child, drawn first by the reversed row.
fn disclosure_trigger(_: &Theme, _: &Look<DisclosureState>) -> Style {
    style()
        .direction(Direction::RowReverse)
        .height(30)
        .gap(6)
        .padding_x(8)
        .align_items(Align::Center)
        .background(Color::Header)
        .hover(style().background(kivact::style::ColorToken::Hover))
}

fn disclosure_indicator(_: &Theme, state: &Look<DisclosureState>) -> Style {
    style()
        .size(16, 16)
        .shrink(0)
        .rotate(if state.open { 0.0 } else { -90.0 })
}

fn disclosure_content(_: &Theme, _: &Look<DisclosureState>) -> Style {
    style().direction(Direction::Column).padding(10)
}
