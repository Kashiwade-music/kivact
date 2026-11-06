//! Role: Key marker shapes for keyframes and similar points on an editor.
use kivact::draw::{Draw, Path};
use kivact::geometry::{Point, Rect};
use kivact::style::Color;

/// The shape a keyframe or marker is drawn with.
///
/// [`Keyframe::glyph`](crate::Keyframe) selects it for a dope sheet's
/// keys; an application painting its own markers calls [`KeyGlyph::paint`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyGlyph {
    /// A diamond, the ordinary keyframe.
    #[default]
    Diamond,
    /// A square, 0.8 of the diamond's radius on each side of the center.
    Square,
    /// A circle of 0.8 of the diamond's radius.
    Circle,
    /// A diamond whose left half is dimmed (35% alpha), for a key that
    /// is only half set, such as a hold or a key on one side.
    HalfDiamond,
}

impl KeyGlyph {
    /// Paint the glyph centered at `center` in `color`. `radius` is the
    /// diamond's half diagonal; the square and the circle use 0.8 of it.
    pub fn paint(self, d: &mut Draw, center: Point, radius: f64, color: Color) {
        let half = |side: f64| {
            Path::new()
                .move_to(Point::new(center.x, center.y - radius))
                .line_to(Point::new(center.x + side * radius, center.y))
                .line_to(Point::new(center.x, center.y + radius))
                .close()
        };
        match self {
            Self::Diamond => d.fill_path(
                &Path::new()
                    .move_to(Point::new(center.x, center.y - radius))
                    .line_to(Point::new(center.x + radius, center.y))
                    .line_to(Point::new(center.x, center.y + radius))
                    .line_to(Point::new(center.x - radius, center.y))
                    .close(),
                color,
            ),
            Self::Square => {
                let h = radius * 0.8;
                d.fill_rect(
                    Rect::new(center.x - h, center.y - h, h * 2.0, h * 2.0),
                    color,
                );
            }
            Self::Circle => {
                let r = radius * 0.8;
                d.rounded_rect(
                    Rect::new(center.x - r, center.y - r, r * 2.0, r * 2.0),
                    r,
                    color,
                );
            }
            Self::HalfDiamond => {
                d.fill_path(&half(-1.0), color.with_alpha(0.35));
                d.fill_path(&half(1.0), color);
            }
        }
    }
}
