//! Role: Draw console empty panels and loading rows through standard recipes.
use crate::Tokens;
use kivact::draw::{Layer, Path, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::style::{Align, BorderStyle, ColorToken, Justify, Style, Theme, pct, style};
use kivact::ui::{EmptyStateRecipes, Look, SkeletonState};

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
