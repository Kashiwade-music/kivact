//! Role: Draw console loading placeholder geometry.
use crate::Tokens;
use kivact::draw::Layer;
use kivact::geometry::Rect;
use kivact::style::{Style, Theme, pct, style};
use kivact::ui::{Look, SkeletonState};

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
