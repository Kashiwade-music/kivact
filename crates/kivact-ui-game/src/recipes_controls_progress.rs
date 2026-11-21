//! Role: Draw the console's progress bar, gauge, ring and disclosure recipes.
//!
//! A flat bar in the variant's tone, optionally [`Striped`](crate::variants::Striped)
//! with moving stripes; the gauge and the ring are the progress variants
//! [`ProgressVariant::Gauge`] and [`ProgressVariant::Ring`]; the disclosure is
//! a ruled band whose chevron turns.
use crate::color::{PRIMARY_HOVER, SURFACE_3};
use crate::tone::ToneExt;
use crate::variants::{GaugeSegments, Striped};
use kivact::draw::{Layer, Path, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::interaction::Tone;
use kivact::style::color::{BORDER, SURFACE, SURFACE_ALT, TEXT_MUTED};
use kivact::style::{Align, Color, FlexDirection, Style, Theme, pct, style};
use kivact::ui::theme::{
    DisclosureRecipeState, DisclosureTheme, Look, ProgressState, ProgressTheme,
};
use kivact::ui::{ControlSize, ProgressVariant};

/// Install the progress and disclosure recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            ProgressTheme::default()
                .track(progress_track)
                .fill(progress_fill)
                .label(progress_label),
        )
        .with(
            DisclosureTheme::default()
                .root(disclosure_root)
                .trigger(disclosure_trigger)
                .indicator(disclosure_indicator)
                .content(disclosure_content),
        )
}

/// A hard-edged bar, 4 px for a small one and 8 px otherwise.
fn progress_track(t: &Theme, look: &Look<ProgressState>, _base: Style) -> Style {
    if let Some(special) = special(t, look) {
        return special;
    }
    style()
        .width(pct(100))
        .height(
            if matches!(look.size, ControlSize::ExtraSmall | ControlSize::Small) {
                4
            } else {
                8
            },
        )
        .min_width(0)
        .clip(true)
        .background(SURFACE_3)
}

fn progress_fill(t: &Theme, look: &Look<ProgressState>, _base: Style) -> Style {
    if let Some(special) = special_fill(look) {
        return special;
    }
    let state = **look;
    let striped = look.variant::<Striped>().is_some();
    let tone = look.variant::<Tone>().unwrap_or(Tone::Info);
    let (fill, stripe) = (tone.color(t), PRIMARY_HOVER);
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
            d.fill_rect(Rect::new(x, 0.0, width, size.height), fill);
            if striped && state.value.is_some() {
                d.layer(
                    Layer::new().clip_rect(Rect::new(x, 0.0, width, size.height)),
                    |d| {
                        let mut x = -24. + state.phase.get() * 17.;
                        while x < width + 24. {
                            d.fill_path(
                                &Path::new()
                                    .move_to(Point::new(x, size.height))
                                    .line_to(Point::new(x + size.height, 0.))
                                    .line_to(Point::new(x + size.height + 6., 0.))
                                    .line_to(Point::new(x + 6., size.height))
                                    .close(),
                                stripe,
                            );
                            x += 12.;
                        }
                    },
                );
            }
        })
}

fn progress_label(t: &Theme, _: &Look<ProgressState>, _base: Style) -> Style {
    style()
        .font_family(
            t.get(&crate::fonts::DISPLAY_FAMILY_VAR)
                .as_str()
                .unwrap_or_default(),
        )
        .font_size(24)
        .font_weight(700)
        .font_style(kivact::style::FontStyle::Italic)
}

/// The gauge or the ring when the progress has one of those variants.
fn special(t: &Theme, s: &Look<ProgressState>) -> Option<Style> {
    match s.variant::<ProgressVariant>() {
        Some(ProgressVariant::Gauge) => Some(gauge(t, s)),
        Some(ProgressVariant::Ring) => Some(ring(t, s)),
        _ => None,
    }
}

fn special_fill(s: &Look<ProgressState>) -> Option<Style> {
    matches!(
        s.variant::<ProgressVariant>(),
        Some(ProgressVariant::Gauge | ProgressVariant::Ring)
    )
    .then(|| style().size(0, 0).min_width(0))
}

fn gauge(t: &Theme, s: &Look<ProgressState>) -> Style {
    let count = s.variant::<GaugeSegments>().map_or(100, |count| count.0);
    let filled = (s.value.unwrap_or(0.) * count as f64).round();
    let color = s.variant::<Tone>().unwrap_or_default().color(t);
    let skew = 9.0 * 12.0_f64.to_radians().tan();
    style()
        .width(count.saturating_mul(15).saturating_sub(3) as f64)
        .height(if count == 0 { 0 } else { 18 })
        .clip(false)
        .background(Color::TRANSPARENT)
        .paint(move |d, _| {
            for i in 0..count {
                let x = (i * 15) as f64;
                d.fill_path(
                    &Path::new()
                        .move_to(Point::new(x + skew, 0.))
                        .line_to(Point::new(x + 12.0 + skew, 0.))
                        .line_to(Point::new(x + 12.0 - skew, 18.))
                        .line_to(Point::new(x - skew, 18.))
                        .close(),
                    if (i as f64) < filled {
                        color
                    } else {
                        SURFACE_3
                    },
                );
            }
        })
}

fn ring(t: &Theme, s: &Look<ProgressState>) -> Style {
    let shown = s.shown;
    let color = s.variant::<Tone>().unwrap_or_default().color(t);
    style()
        .size(88, 88)
        .clip(false)
        .background(Color::TRANSPARENT)
        .flex_direction(FlexDirection::ColumnReverse)
        .align_items(Align::Center)
        .justify_content(kivact::style::Justify::Center)
        .gap(0)
        .paint(move |d, _| {
            let center = Point::new(44., 44.);
            d.stroke_path(
                &Path::new().arc(center, 36., 0., 359.9, false),
                &Stroke::new(8),
                SURFACE_3,
            );
            let end = -90. + shown.get() * 360.;
            if end > -90. {
                d.stroke_path(
                    &Path::new().arc(center, 36., -90., end, false),
                    &Stroke::new(8),
                    color,
                );
            }
            d.stroke_path(
                &Path::new().arc(center, 36., 0., 359.9, false),
                &Stroke::new(9).dash([1.2, 8.62]),
                SURFACE,
            );
        })
}

fn disclosure_root(_: &Theme, _: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style().flex_direction(FlexDirection::Column)
}

/// The chevron is the trigger's last child, drawn first by the reversed row.
fn disclosure_trigger(_: &Theme, _: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::RowReverse)
        .height(30)
        .padding_x(10)
        .gap(7)
        .align_items(Align::Center)
        .font_size(11.5)
        .font_weight(700)
        .background(SURFACE_ALT)
        .border_top(1, BORDER)
        .border_bottom(1, BORDER)
}

fn disclosure_indicator(_: &Theme, state: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style()
        .size(10, 10)
        .flex_shrink(0.)
        .color(TEXT_MUTED)
        .rotate(if state.open { 0. } else { -90. })
}

fn disclosure_content(_: &Theme, _: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style()
        .enter_size(kivact::view::Orientation::Vertical)
        .exit_size(kivact::view::Orientation::Vertical)
        .flex_direction(FlexDirection::Column)
}
