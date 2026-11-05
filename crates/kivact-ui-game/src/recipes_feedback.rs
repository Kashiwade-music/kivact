//! Role: Draw console empty panels and loading rows through standard recipes.
use crate::Tokens;
use crate::display::ToneExt;
use kivact::draw::{Layer, Path, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::style::{Align, BorderStyle, ColorToken, Direction, Justify, Style, Theme, pct, style};
use kivact::ui::{AlertRecipes, AlertState, AlertVariant, EmptyStateRecipes, Look, SkeletonState};

/// A console loading row with an avatar and two caption placeholders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkeletonVariant {
    Rows,
}
impl kivact::ui::Variant for SkeletonVariant {}

pub(crate) fn skeleton(t: &Theme, s: &Look<SkeletonState>) -> Style {
    if s.variant::<SkeletonVariant>().is_none() {
        let base = style()
            .min_height(16)
            .width(pct(100))
            .background(t.ext::<Tokens>().surface_3)
            .opacity(s.pulse);
        return match s.shape {
            kivact::ui::SkeletonShape::Circle => base.size(36, 36).radius(999),
            kivact::ui::SkeletonShape::Rounded => base.radius(t.radius().md),
            kivact::ui::SkeletonShape::Text => base.radius(t.radius().sm),
            kivact::ui::SkeletonShape::Rect => base.radius(0),
        };
    }
    let phase = (1.0 - s.pulse) * 2.0;
    style().height(36).width(pct(100)).paint(move |d, _| {
        for rect in [
            Rect::new(0., 0., 36., 36.),
            Rect::new(48., 4., 160., 10.),
            Rect::new(48., 22., 230., 10.),
        ] {
            let layer = if rect.x == 0.0 {
                Layer::new().area(rect).clip(&crate::cut_corner(8.))
            } else {
                Layer::new().clip_rect(rect)
            };
            d.layer(layer, |d| {
                let p = d.theme().ext::<Tokens>();
                d.fill_rect(rect, p.surface_3);
                let x = rect.x + rect.width * (phase * 3.0 - 1.0);
                d.fill_rect(
                    Rect::new(x, rect.y, rect.width * 0.3, rect.height),
                    kivact::draw::linear_gradient(100).colors([
                        p.surface_3,
                        d.theme().colors().surface_alt,
                        p.surface_3,
                    ]),
                );
            });
        }
    })
}
pub(crate) fn empty_state(r: EmptyStateRecipes) -> EmptyStateRecipes {
    r.root(|_| {
        move |_, _| {
            style()
                .direction(Direction::Column)
                .width(pct(100))
                .padding_y(28)
                .padding_x(16)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .gap(8)
                .border(1.5, ColorToken::Border)
                .border_style(BorderStyle::Dashed)
        }
    })
    .title(|_| {
        move |t, _| {
            style()
                .font_family(t.ext::<Tokens>().display_family.as_str())
                .font_size(20)
                .weight(700)
                .italic(true)
        }
    })
    .description(|_| {
        move |_, _| {
            style()
                .font_size(12.5)
                .max_width(300)
                .color(ColorToken::TextMuted)
        }
    })
    .icon(|_| {
        move |_, _| {
            style().size(64, 64).paint(|d, _| {
                let color = d.theme().ext::<Tokens>().text_3;
                d.stroke_path(
                    &Path::new().arc(Point::new(32., 32.), 28., 0., 359.9, false),
                    &Stroke::new(2.).dash([10., 4., 3., 4.]),
                    color,
                );
                d.stroke_rect(Rect::new(22., 26., 20., 16.), 2., color);
                d.polyline(
                    [
                        Point::new(22., 26.),
                        Point::new(26., 21.),
                        Point::new(38., 21.),
                        Point::new(42., 26.),
                    ],
                    &Stroke::new(2.),
                    color,
                );
                d.line(
                    Point::new(28., 33.),
                    Point::new(36., 33.),
                    2.5,
                    ColorToken::Accent,
                );
            })
        }
    })
    .actions(|base| move |t, s| base(t, s).align_items(Align::Center))
}

pub(crate) fn alert(r: AlertRecipes) -> AlertRecipes {
    r.root(|_| alert_root)
        .title(|_| alert_title)
        .description(|_| {
            move |_, _| {
                style()
                    .grid_column(2)
                    .grid_row(2)
                    .font_size(12.5)
                    .color(ColorToken::TextMuted)
            }
        })
        .icon(|_| alert_icon)
        .actions(|_| {
            move |_, s| {
                if banner(s) {
                    style()
                        .grid_column(3)
                        .grid_row(1)
                        .align_items(Align::Center)
                } else {
                    style().grid_column(2).grid_row(3)
                }
            }
        })
        .content(|_| {
            move |_, s| {
                if banner(s) {
                    style()
                        .grid_column(2)
                        .grid_row(1)
                        .font_size(12.5)
                        .color(ColorToken::AccentText)
                        .grow(1)
                } else {
                    style().grid_column(2).grid_row(4)
                }
            }
        })
        .dismiss(|_| {
            move |_, _| {
                style()
                    .grid_column(3)
                    .grid_row((1, 3))
                    .align_self(Align::Start)
                    .size(32, 32)
                    .padding(0)
                    .background(kivact::style::Color::TRANSPARENT)
                    .color(ColorToken::TextMuted)
                    .border_width(0)
                    .shadow(kivact::draw::Shadow::new(
                        0,
                        0,
                        0,
                        kivact::style::Color::TRANSPARENT,
                    ))
            }
        })
        .toggle(|base| move |t, s| base(t, s).grid_column(2).grid_row(4))
}
fn banner(s: &Look<AlertState>) -> bool {
    s.variant::<AlertVariant>() == Some(&AlertVariant::Banner)
}
fn alert_root(t: &Theme, s: &Look<AlertState>) -> Style {
    let base = style().display(kivact::style::Display::Grid).radius(0);
    if banner(s) {
        base.width(pct(100))
            .height(44)
            .padding_right(14)
            .gap(14)
            .grid_template_columns((
                kivact::style::auto(),
                kivact::style::Track::fr(1),
                kivact::style::auto(),
            ))
            .align_items(Align::Center)
            .background(t.ext::<Tokens>().navy)
            .color(ColorToken::AccentText)
    } else {
        let base = if s.dismissible {
            base.grid_template_columns((18, kivact::style::Track::fr(1), 32))
        } else {
            base.grid_template_columns((18, kivact::style::Track::fr(1)))
        };
        base.padding_y(12)
            .padding_x(14)
            .column_gap(12)
            .row_gap(2)
            .align_items(Align::Start)
            .background(s.tone.weak(t))
            .border(1, s.tone.color(t).mix(t.colors().surface, 0.65))
            .border_left(4, s.tone.color(t))
    }
}
fn alert_title(t: &Theme, s: &Look<AlertState>) -> Style {
    if banner(s) {
        banner_tape(t, s)
            .grid_column(1)
            .grid_row(1)
            .font_family(t.ext::<Tokens>().mono_family.as_str())
            .font_size(10)
            .weight(800)
            .text_wrap(false)
            .padding_x(18)
            .padding_y(14.25)
    } else {
        style().grid_column(2).grid_row(1).font_size(13).weight(700)
    }
}
fn alert_icon(t: &Theme, s: &Look<AlertState>) -> Style {
    style()
        .grid_column(1)
        .grid_row((1, 3))
        .size(18, 18)
        .color(s.tone.color(t))
}
fn banner_tape(t: &Theme, s: &Look<AlertState>) -> Style {
    let stripe = t.ext::<Tokens>().tape_stripe;
    let color = if s.tone == kivact::interaction::Tone::Warning {
        t.ext::<Tokens>().yellow
    } else {
        s.tone.color(t)
    };
    style()
        .grid_column(1)
        .grid_row(1)
        .height(pct(100))
        .padding_x(12)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(t.ext::<Tokens>().navy)
        .paint(move |d, s| {
            d.layer(Layer::new().clip_rect(s.to_rect()), |d| {
                d.fill_rect(s.to_rect(), color);
                let mut x = -44.;
                while x < s.width + 44. {
                    d.fill_path(
                        &Path::new()
                            .move_to(Point::new(x, s.height))
                            .line_to(Point::new(x + s.height, 0.))
                            .line_to(Point::new(x + s.height + 8., 0.))
                            .line_to(Point::new(x + 8., s.height))
                            .close(),
                        stripe,
                    );
                    x += 16.;
                }
                d.fill_rect(
                    Rect::new(12., (s.height - 23.5) / 2., s.width - 24., 23.5),
                    color,
                );
            })
        })
}
