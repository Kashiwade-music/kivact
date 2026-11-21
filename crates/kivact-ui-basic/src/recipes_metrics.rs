//! Role: Lyra circular and segmented progress recipes, and the heading recipe.
//!
//! The ring and segmented looks of progress.tsx, which the bar of
//! `recipes_controls` hands over to, and the heading levels of typography.tsx.

use crate::color::{BORDER, FOREGROUND, MUTED, PRIMARY};
use kivact::draw::{LineCap, Path, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::style::{Align, Justify, Position, Style, Theme, pct, style};
use kivact::ui::theme::{HeadingState, HeadingTheme, Look, ProgressState};
use kivact::ui::{ControlSize, ProgressVariant};

fn height(s: &Look<ProgressState>) -> f64 {
    match s.size {
        ControlSize::ExtraSmall => 3.0,
        ControlSize::Small => 4.0,
        ControlSize::Medium => 6.0,
        ControlSize::Large => 10.0,
    }
}

/// The track of a ring: a muted circle stroke; of any other variant: twenty
/// muted segments across the height of the bar.
pub(crate) fn track(_: &Theme, s: &Look<ProgressState>, _base: Style) -> Style {
    if s.variant::<ProgressVariant>() == Some(ProgressVariant::Ring) {
        style()
            .size(height(s) * 12.0, height(s) * 12.0)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .paint(move |d, size| {
                let r = size.width.min(size.height) * 0.4;
                d.stroke_path(
                    &Path::new().arc(size.to_rect().center(), r, 0.0, 359.99, false),
                    &Stroke::new(r / 5.0),
                    MUTED,
                );
            })
    } else {
        style()
            .width(pct(100.0))
            .height(height(s) * 3.0)
            .min_width(0.0)
            .paint(move |d, size| {
                for i in 0..20 {
                    d.fill_rect(
                        Rect::new(
                            i as f64 * size.width / 20.0,
                            0.0,
                            size.width / 20.0 * 0.75,
                            size.height,
                        ),
                        MUTED,
                    );
                }
            })
    }
}

/// The fill of a ring: an arc from the top, the length of the value (or the
/// indeterminate sweep); of any other variant: the segments the value covers.
pub(crate) fn fill(_: &Theme, s: &Look<ProgressState>, _base: Style) -> Style {
    let state = **s;
    let ring = s.variant::<ProgressVariant>() == Some(ProgressVariant::Ring);
    style()
        .position(Position::Absolute)
        .inset(0.0)
        .paint(move |d, size| {
            let (start, len) = if state.value.is_some() {
                (0.0, state.shown.get().clamp(0.0, 1.0))
            } else {
                state.sweep()
            };
            if ring {
                let r = size.width.min(size.height) * 0.4;
                if len > 0.0 {
                    d.stroke_path(
                        &Path::new().arc(
                            Point::new(size.width / 2.0, size.height / 2.0),
                            r,
                            -90.0 + start * 360.0,
                            -90.0 + (start + len.min(0.9999)) * 360.0,
                            false,
                        ),
                        &Stroke::new(r / 5.0).cap(LineCap::Round),
                        PRIMARY,
                    );
                }
            } else {
                for i in 0..20 {
                    let f = (i as f64 + 0.5) / 20.0;
                    if f >= start && f <= start + len {
                        d.fill_rect(
                            Rect::new(
                                i as f64 * size.width / 20.0,
                                0.0,
                                size.width / 20.0 * 0.75,
                                size.height,
                            ),
                            PRIMARY,
                        );
                    }
                }
            }
        })
}

/// The heading of a level: 36, 30, 24, 20, 16 and 14 px; level 2 has a rule
/// under it.
fn heading(_: &Theme, s: &Look<HeadingState>, _base: Style) -> Style {
    let base = style().color(FOREGROUND);
    match s.level {
        1 => base
            .font_size(36.0)
            .line_height(40.0 / 36.0)
            .font_weight(800)
            .letter_spacing(-0.9),
        2 => base
            .font_size(30.0)
            .line_height(36.0 / 30.0)
            .font_weight(600)
            .letter_spacing(-0.75)
            .padding_bottom(8.0)
            .border_bottom(1.0, BORDER),
        3 => base
            .font_size(24.0)
            .line_height(32.0 / 24.0)
            .font_weight(600)
            .letter_spacing(-0.6),
        4 => base
            .font_size(20.0)
            .line_height(28.0 / 20.0)
            .font_weight(600)
            .letter_spacing(-0.5),
        5 => base.font_size(16.0).font_weight(600),
        _ => base.font_size(14.0).font_weight(600),
    }
}

/// Installs the heading.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(HeadingTheme::default().root(heading))
}
