//! Role: Style Soft sliders and progress bars using their standard value parts.
use crate::Tokens;
use kivact::draw::Rect;
use kivact::interaction::Tone;
use kivact::motion::Easing;
use kivact::platform::reduced_motion;
use kivact::style::{Color, Cursor, Overflow, Style, Theme, pct, style};
use kivact::ui::{AxisState, Look, ProgressRecipes, ProgressState, SliderRecipes};
use kivact::view::{Orientation, ReadingDirection, current_direction};

use super::recipe_style::{fast, overlay, ring};

pub(super) fn slider(r: SliderRecipes) -> SliderRecipes {
    r.track(|_| track).range(|_| range).thumb(|_| thumb)
}

fn slider_band(t: &Tokens, s: &Look<AxisState>, ink: Color) -> Style {
    let thickness = t.spacing.s1;
    let radius = t.radius.full;
    let vertical = s.orientation == Orientation::Vertical;
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

fn track(theme: &Theme, s: &Look<AxisState>) -> Style {
    let t = theme.get::<Tokens>();
    let base = slider_band(&t, s, t.colors.track)
        .cursor(Cursor::Pointer)
        .shrink(0)
        .disabled(
            style()
                .opacity(t.sizes.disabled_opacity)
                .cursor(Cursor::Default),
        );
    let side = t.spacing.s5;
    if s.orientation == Orientation::Vertical {
        base.width(side).height(t.spacing.s10 * 4.0)
    } else {
        base.width(pct(100)).height(side)
    }
}

fn range(theme: &Theme, s: &Look<AxisState>) -> Style {
    let t = theme.get::<Tokens>();
    slider_band(&t, s, t.colors.accent)
}

fn thumb(theme: &Theme, _: &Look<AxisState>) -> Style {
    let t = theme.get::<Tokens>();
    let c = &t.colors;
    let side = t.spacing.s5;
    // TODO(soft-core: C-09): Keep the standard desktop pointer behavior
    // until the touch lift, relative tracking and coast are available.
    let base = style()
        .size(side, side)
        .shrink(0)
        .radius(t.radius.full)
        .cursor(Cursor::Grab)
        .background(c.accent)
        .hover(style().background(c.accent_hover))
        .pressed(style().background(overlay(theme, c.accent, c.overlay_pressed)))
        .disabled(style().background(c.bg_muted).cursor(Cursor::Default))
        .focus_visible(ring(&t, c.focus_ring));
    fast(&t, base)
}

pub(super) fn progress(r: ProgressRecipes) -> ProgressRecipes {
    r.track(|_| progress_track).fill(|_| progress_fill)
}

fn progress_track(theme: &Theme, _: &Look<ProgressState>) -> Style {
    let t = theme.get::<Tokens>();
    style()
        .width(pct(100))
        .min_width(t.spacing.s12)
        .height(t.spacing.s2)
        .radius(t.radius.full)
        .background(t.colors.border_emphasized)
        .overflow(Overflow::Hidden)
        .clip(true)
}

fn progress_fill(theme: &Theme, s: &Look<ProgressState>) -> Style {
    let t = theme.get::<Tokens>();
    let c = &t.colors;
    let ink = match s.variant::<Tone>().copied() {
        Some(Tone::Success) => c.success,
        Some(Tone::Warning) => c.warning,
        Some(Tone::Danger) => c.error,
        Some(Tone::Neutral) => c.text_disabled,
        _ => c.accent,
    };
    let (shown, phase) = (s.shown, s.phase);
    let indeterminate = s.value.is_none();
    let rtl = current_direction() == ReadingDirection::Rtl;
    style().size(pct(100), pct(100)).paint(move |d, size| {
        // TODO(soft-core: C-41): Configure the standard progress clock and
        // determinate transition, and expose its reduced-motion pulse.
        let (start, length) = if indeterminate {
            const BAND: f32 = 0.4;
            if reduced_motion() {
                (0.0, 1.0)
            } else {
                let ease = Easing::CubicBezier(0.42, 0.0, 0.58, 1.0);
                (-BAND + (1.0 + BAND) * ease.apply(phase.get()), BAND)
            }
        } else {
            (0.0, shown.get().clamp(0.0, 1.0) as f32)
        };
        if length > 0.0 {
            let start = if rtl { 1.0 - start - length } else { start };
            let rect = Rect::new(start * size.width, 0.0, length * size.width, size.height);
            d.rounded_rect(rect, size.height / 2.0, ink);
        }
    })
}
