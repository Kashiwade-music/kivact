//! Role: Style Astryx Neutral sliders and progress bars using their standard value parts.
use crate::color as c;
use crate::metrics as m;
use crate::radius as r;
use crate::recipe_parts::{fast, overlay, ring};
use crate::space as s;
use crate::type_scale as t;
use kivact::geometry::Rect;
use kivact::interaction::Tone;
use kivact::motion::Easing;
use kivact::platform::is_motion_reduced;
use kivact::style::{Color, Cursor, Overflow, Style, Theme, pct, style};
use kivact::ui::ProgressVariant;
use kivact::ui::theme::{AxisState, Look, ProgressState, ProgressTheme, SliderTheme};
use kivact::view::{Orientation, ReadingDirection, use_direction};

/// A rounded band across a slider track, `ink`-colored, `thickness` wide.
fn slider_band(theme: &Theme, look: &Look<AxisState>, ink: Color) -> Style {
    let thickness = theme.get(&s::SPACE_S1_VAR);
    let radius = theme.get(&r::RADIUS_FULL_VAR);
    let vertical = look.orientation == Orientation::Vertical;
    style().paint(move |d, size| {
        let rect = if vertical {
            Rect::new((size.width - thickness) / 2.0, 0.0, thickness, size.height)
        } else {
            Rect::new(0.0, (size.height - thickness) / 2.0, size.width, thickness)
        };
        if rect.width > 0.0 && rect.height > 0.0 {
            d.rounded_rect(rect, radius, ink);
        }
    })
}

fn track(theme: &Theme, look: &Look<AxisState>, _base: Style) -> Style {
    let base = slider_band(theme, look, c::TRACK)
        .cursor(Cursor::Pointer)
        .flex_shrink(0)
        .disabled(style().opacity(m::DISABLED_OPACITY).cursor(Cursor::Default));
    let side = theme.get(&s::SPACE_S5_VAR);
    if look.orientation == Orientation::Vertical {
        base.width(side).height(theme.get(&s::SPACE_S10_VAR) * 4.0)
    } else {
        base.width(pct(100)).height(side)
    }
}

fn range(theme: &Theme, look: &Look<AxisState>, _base: Style) -> Style {
    slider_band(theme, look, c::ACCENT)
}

fn thumb(theme: &Theme, _look: &Look<AxisState>, _base: Style) -> Style {
    let side = theme.get(&s::SPACE_S5_VAR);
    let base = style()
        .size(side, side)
        .flex_shrink(0)
        .border_radius(r::RADIUS_FULL)
        .cursor(Cursor::Grab)
        .background(c::ACCENT)
        .hover(style().background(c::ACCENT_HOVER))
        .active(style().background(overlay(theme, c::ACCENT, c::OVERLAY_PRESSED)))
        .disabled(style().background(c::BG_MUTED).cursor(Cursor::Default))
        .focus_visible(ring(Color::var(&c::FOCUS_RING_VAR)));
    fast(base)
}

/// Whether a bar uses the standard bar look: no ring and no gauge variant.
fn is_bar(look: &Look<ProgressState>) -> bool {
    !look
        .variant::<ProgressVariant>()
        .is_some_and(|variant| variant != ProgressVariant::Bar)
}

fn progress_track(theme: &Theme, look: &Look<ProgressState>, base: Style) -> Style {
    if !is_bar(look) {
        return base;
    }
    style()
        .width(pct(100))
        .min_width(theme.get(&s::SPACE_S12_VAR))
        .height(theme.get(&s::SPACE_S2_VAR))
        .border_radius(r::RADIUS_FULL)
        .background(c::BORDER_EMPHASIZED)
        .overflow(Overflow::Hidden)
        .clip(true)
}

fn progress_fill(_theme: &Theme, look: &Look<ProgressState>, base: Style) -> Style {
    if !is_bar(look) {
        return base;
    }
    let ink = match look.variant::<Tone>() {
        Some(Tone::Success) => c::SUCCESS,
        Some(Tone::Warning) => c::WARNING,
        Some(Tone::Danger) => c::ERROR,
        Some(Tone::Neutral) => c::TEXT_DISABLED,
        _ => c::ACCENT,
    };
    let (shown, phase) = (look.shown, look.phase);
    let indeterminate = look.value.is_none();
    let rtl = use_direction() == ReadingDirection::Rtl;
    style().size(pct(100), pct(100)).paint(move |d, size| {
        let ease = Easing::CubicBezier(0.42, 0.0, 0.58, 1.0);
        let (start, length, ink) = if indeterminate {
            const BAND: f64 = 0.4;
            let phase = phase.get();
            if is_motion_reduced().get() {
                // The 4000 ms cycle alternates two 2000 ms opacity passes.
                let down_and_back = 1.0 - (2.0 * phase - 1.0).abs();
                let opacity = 1.0 - 0.16 * ease.apply(down_and_back);
                (0.0, 1.0, ink.with_alpha(opacity))
            } else {
                (-BAND + (1.0 + BAND) * ease.apply(phase), BAND, ink)
            }
        } else {
            (0.0, shown.get().clamp(0.0, 1.0), ink)
        };
        if length > 0.0 {
            let start = if rtl { 1.0 - start - length } else { start };
            let rect = Rect::new(start * size.width, 0.0, length * size.width, size.height);
            d.rounded_rect(rect, size.height / 2.0, ink);
        }
    })
}

fn progress_label(_theme: &Theme, _look: &Look<ProgressState>, _base: Style) -> Style {
    style().font_size(t::BODY.size_px).color(c::TEXT_PRIMARY)
}

/// Install the kit's recipes of the slider and the progress bar.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            SliderTheme::default()
                .track(track)
                .range(range)
                .thumb(thumb),
        )
        .with(
            ProgressTheme::default()
                .track(progress_track)
                .fill(progress_fill)
                .label(progress_label),
        )
}
