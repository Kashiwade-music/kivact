//! Role: Style Soft sliders and progress bars using their standard value parts.
use crate::Tokens;
use kivact::geometry::Rect;
use kivact::interaction::Tone;
use kivact::motion::Easing;
use kivact::platform::is_motion_reduced;
use kivact::style::Color;
use kivact::style::Cursor;
use kivact::style::Overflow;
use kivact::style::Style;
use kivact::style::Theme;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::AxisState;
use kivact::ui::Look;
use kivact::ui::ProgressRecipes;
use kivact::ui::ProgressState;
use kivact::ui::SliderRecipes;
use kivact::view::Orientation;
use kivact::view::ReadingDirection;
use kivact::view::use_direction;

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
    let t = theme.ext::<Tokens>();
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
    let t = theme.ext::<Tokens>();
    slider_band(&t, s, t.colors.accent)
}

fn thumb(theme: &Theme, _: &Look<AxisState>) -> Style {
    let t = theme.ext::<Tokens>();
    let c = &t.colors;
    let side = t.spacing.s5;
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
    r.track(|prev| {
        move |t: &Theme, s: &Look<ProgressState>| {
            if s.variant::<kivact::ui::ProgressVariant>()
                .is_some_and(|v| *v != kivact::ui::ProgressVariant::Bar)
            {
                prev(t, s)
            } else {
                progress_track(t, s)
            }
        }
    })
    .fill(|prev| {
        move |t: &Theme, s: &Look<ProgressState>| {
            if s.variant::<kivact::ui::ProgressVariant>()
                .is_some_and(|v| *v != kivact::ui::ProgressVariant::Bar)
            {
                prev(t, s)
            } else {
                progress_fill(t, s)
            }
        }
    })
    .label(|_| {
        |t: &Theme, _: &Look<ProgressState>| {
            style()
                .font_size(t.ext::<Tokens>().typography.body.size_px)
                .color(t.ext::<Tokens>().colors.text_primary)
        }
    })
}

fn progress_track(theme: &Theme, _: &Look<ProgressState>) -> Style {
    let t = theme.ext::<Tokens>();
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
    let t = theme.ext::<Tokens>();
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
    let rtl = use_direction().get() == ReadingDirection::Rtl;
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
