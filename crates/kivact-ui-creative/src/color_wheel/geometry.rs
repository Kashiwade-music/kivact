//! Role: Shared color picker painting and hit geometry.
use super::{color_from_hsv, color_to_hsv};
use crate::editors::shared::metrics::{PICKER_ALPHA, PICKER_GAP};
use kivact::draw::{Draw, Path};
use kivact::geometry::{Point, Rect, Size};
use kivact::style::Color;
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Channel {
    Hue,
    Sv,
    Alpha,
}

pub(super) struct Geometry {
    center: Point,
    radius: f64,
    square: Rect,
    alpha: Rect,
}
impl Geometry {
    pub(super) fn new(size: Size, has_alpha: bool) -> Self {
        let side = size.width.max(1.0).min(
            (size.height
                - if has_alpha {
                    PICKER_ALPHA + PICKER_GAP
                } else {
                    0.0
                })
            .max(1.0),
        );
        let radius = side * 0.48;
        let center = Point::new(side * 0.5, side * 0.5);
        let half = radius * 0.58;
        Self {
            center,
            radius,
            square: Rect::new(center.x - half, center.y - half, half * 2.0, half * 2.0),
            alpha: Rect::new(
                0.0,
                size.height - PICKER_ALPHA,
                size.width,
                if has_alpha { PICKER_ALPHA } else { 0.0 },
            ),
        }
    }
    pub(super) fn update(
        &self,
        channel: Channel,
        point: Point,
        color: Color,
        hue: f64,
    ) -> (Color, f64) {
        let (_, mut s, mut v) = color_to_hsv(color);
        let mut alpha = color.components()[3];
        let mut hue = hue;
        match channel {
            Channel::Hue => {
                hue = (point.y - self.center.y)
                    .atan2(point.x - self.center.x)
                    .to_degrees()
                    .rem_euclid(360.0)
            }
            Channel::Sv => {
                s = ((point.x - self.square.x) / self.square.width).clamp(0.0, 1.0);
                v = 1.0 - ((point.y - self.square.y) / self.square.height).clamp(0.0, 1.0);
            }
            Channel::Alpha => alpha = (point.x / self.alpha.width.max(1.0)).clamp(0.0, 1.0),
        }
        (color_from_hsv(hue, s, v, alpha), hue)
    }
}

impl Channel {
    pub(super) fn id(self) -> kivact::draw::RegionId {
        match self {
            Self::Hue => kivact::draw::RegionId(1),
            Self::Sv => kivact::draw::RegionId(2),
            Self::Alpha => kivact::draw::RegionId(3),
        }
    }
}
impl Geometry {
    pub(super) fn regions(&self, draw: &mut Draw) {
        draw.region(
            Channel::Hue.id(),
            Rect::new(
                self.center.x - self.radius,
                self.center.y - self.radius,
                self.radius * 2.0,
                self.radius * 2.0,
            ),
        );
        draw.region(Channel::Sv.id(), self.square);
        if self.alpha.height > 0.0 {
            draw.region(Channel::Alpha.id(), self.alpha);
        }
    }
}
pub(super) fn paint(draw: &mut Draw, geometry: Geometry, color: Color, hue: f64) {
    let Geometry {
        center,
        radius,
        square,
        alpha,
    } = geometry;
    for index in 0..120 {
        let a = index as f64 * std::f64::consts::TAU / 120.0;
        let b = (index + 1) as f64 * std::f64::consts::TAU / 120.0;
        let inner = radius * 0.86;
        draw.fill_path(
            &Path::new()
                .move_to(Point::new(
                    center.x + inner * a.cos(),
                    center.y + inner * a.sin(),
                ))
                .line_to(Point::new(
                    center.x + radius * a.cos(),
                    center.y + radius * a.sin(),
                ))
                .line_to(Point::new(
                    center.x + radius * b.cos(),
                    center.y + radius * b.sin(),
                ))
                .line_to(Point::new(
                    center.x + inner * b.cos(),
                    center.y + inner * b.sin(),
                ))
                .close(),
            color_from_hsv(f64::from(index) * 3.0, 1.0, 1.0, 1.0),
        );
    }
    draw.fill_rect(
        square,
        kivact::draw::linear_gradient(90)
            .colors([Color::WHITE, color_from_hsv(hue, 1.0, 1.0, 1.0)]),
    );
    draw.fill_rect(
        square,
        kivact::draw::linear_gradient(180).colors([Color::TRANSPARENT, Color::BLACK]),
    );
    let (_, s, v) = color_to_hsv(color);
    marker(
        draw,
        Point::new(
            square.x + s * square.width,
            square.y + (1.0 - v) * square.height,
        ),
    );
    let angle = hue.to_radians();
    marker(
        draw,
        Point::new(
            center.x + radius * 0.93 * angle.cos(),
            center.y + radius * 0.93 * angle.sin(),
        ),
    );
    if alpha.height <= 0.0 {
        return;
    }
    for x in 0..(alpha.width / 6.0).ceil() as usize {
        for y in 0..2 {
            draw.fill_rect(
                Rect::new(x as f64 * 6.0, alpha.y + y as f64 * 6.0, 6.0, 6.0),
                crate::color::checker_color((x + y) % 2 == 0),
            );
        }
    }
    draw.fill_rect(
        alpha,
        kivact::draw::linear_gradient(90).colors([color.with_alpha(0.0), color.with_alpha(1.0)]),
    );
    marker(
        draw,
        Point::new(
            color.components()[3] * alpha.width,
            alpha.y + alpha.height / 2.0,
        ),
    );
}
fn marker(draw: &mut Draw, point: Point) {
    draw.circle(point, 5.0, Color::BLACK);
    draw.circle(point, 3.0, Color::WHITE);
}
