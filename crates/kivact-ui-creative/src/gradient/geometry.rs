//! Role: Stable sorted gradient display and shared ramp hit geometry.
use super::data::{GradientStop, position_value};
use kivact::draw::{Draw, Rect, Size, linear_gradient};
use kivact::style::Color;
#[derive(Clone, Copy)]
pub(super) struct Ramp {
    pub(super) width: f64,
}
impl Ramp {
    pub(super) fn new(size: Size) -> Self {
        Self {
            width: (size.width - 16.0).max(1.0),
        }
    }
    pub(super) fn hit(self, position: f64) -> Rect {
        Rect::new(
            1.0 + (position_value(position) * self.width),
            24.0,
            14.0,
            20.0,
        )
    }
    pub(super) fn paint(
        self,
        draw: &mut Draw,
        stops: &[GradientStop],
        selected: Option<usize>,
        display: &dyn Fn(Color) -> Color,
    ) {
        let mut sorted = stops.iter().enumerate().collect::<Vec<_>>();
        sorted
            .sort_by(|a, b| position_value(a.1.position).total_cmp(&position_value(b.1.position)));
        draw.fill_rect(Rect::new(8.0, 0.0, self.width, 24.0), Color::TRANSPARENT);
        if let Some((_, first)) = sorted.first() {
            draw.fill_rect(
                Rect::new(8.0, 0.0, position_value(first.position) * self.width, 24.0),
                display(first.color),
            );
            for pair in sorted.windows(2) {
                let a = position_value(pair[0].1.position);
                let b = position_value(pair[1].1.position);
                if b > a {
                    draw.fill_rect(
                        Rect::new(8.0 + (a * self.width), 0.0, (b - a) * self.width, 24.0),
                        linear_gradient(90)
                            .colors([display(pair[0].1.color), display(pair[1].1.color)]),
                    );
                }
            }
            let (_, last) = sorted.last().expect("invariant: nonempty gradient");
            let at = position_value(last.position);
            draw.fill_rect(
                Rect::new(8.0 + (at * self.width), 0.0, (1.0 - at) * self.width, 24.0),
                display(last.color),
            );
        }
        for (index, stop) in sorted {
            let hit = self.hit(stop.position);
            draw.fill_rect(
                hit,
                if selected == Some(index) {
                    Color::WHITE
                } else {
                    Color::BLACK
                },
            );
            draw.fill_rect(
                Rect::new(hit.x + 2.0, hit.y + 2.0, hit.width - 4.0, hit.height - 4.0),
                display(stop.color),
            );
        }
    }
}
