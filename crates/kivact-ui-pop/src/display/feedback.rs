//! Role: Render Pop loading and empty feedback states.

use crate::{Hue, Tokens};
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::state::Value;
use kivact::state::{IntoText, IntoValue};
use kivact::style::{Align, ColorToken, style};
use kivact::view::{IntoView, View};
use kivact::view::{Node, NodeBuilder, column, node, text};

#[track_caller]
pub fn empty_state(title: impl IntoText) -> EmptyState {
    EmptyState {
        root: column(()),
        title: title.into_text_value(),
        description: String::new().into_value(),
        action: None,
    }
}
/// Presentation builder returned by [`empty_state`].
pub struct EmptyState {
    root: Node,
    title: Value<String>,
    description: Value<String>,
    action: Option<kivact::view::View>,
}
impl EmptyState {
    /// Supporting text; empty by default.
    pub fn description(mut self, value: impl IntoText) -> Self {
        self.description = value.into_text_value();
        self
    }
    /// Optional content built once when the state mounts.
    pub fn content(mut self, value: impl IntoView) -> Self {
        self.action = Some(value.into_view());
        self
    }
}
impl NodeBuilder for EmptyState {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for EmptyState {
    fn into_view(self) -> View {
        let title = self.title;
        let description = self.description;
        let action = self.action;

        self.root
            .child((
                node().base_style(style().size(160, 110)).draw(|d, _| {
                    let p = d.theme().ext::<Tokens>();
                    let mut halo = Path::new();
                    for i in 0..=64 {
                        let a = std::f64::consts::TAU * i as f64 / 64.0;
                        let point = Point::new(80.0 + 34.0 * a.cos(), 22.0 + 9.0 * a.sin());
                        halo = if i == 0 {
                            halo.move_to(point)
                        } else {
                            halo.line_to(point)
                        };
                    }
                    d.stroke_path(
                        &halo,
                        &Stroke::new(4.0).cap(LineCap::Round).join(LineJoin::Round),
                        p.blue,
                    );
                    d.rounded_rect(
                        Rect {
                            x: 38.0,
                            y: 42.0,
                            width: 84.0,
                            height: 56.0,
                        },
                        12.0,
                        p.hue(Hue::Blue).0,
                    );
                    d.line(
                        Point::new(38.0, 60.0),
                        Point::new(122.0, 60.0),
                        2.0,
                        p.line_strong,
                    );
                    for x in [66.0, 94.0] {
                        d.rounded_rect(
                            Rect {
                                x: x - 3.5,
                                y: 74.5,
                                width: 7.0,
                                height: 7.0,
                            },
                            3.5,
                            p.text_faint,
                        );
                    }
                    d.stroke_path(
                        &Path::new()
                            .move_to(Point::new(72.0, 88.0))
                            .quad_to(Point::new(80.0, 82.0), Point::new(88.0, 88.0)),
                        &Stroke::new(2.5).cap(LineCap::Round).join(LineJoin::Round),
                        p.text_faint,
                    );
                    for (x, y, r, color) in
                        [(132.0, 40.0, 4.0, p.yellow), (27.0, 70.0, 3.0, p.cyan)]
                    {
                        d.fill_path(
                            &Path::new()
                                .move_to(Point::new(x, y - r * 2.0))
                                .line_to(Point::new(x + r, y))
                                .line_to(Point::new(x, y + r * 2.0))
                                .line_to(Point::new(x - r, y))
                                .close(),
                            color,
                        );
                    }
                }),
                text(title).base_style(style().font_size(16).weight(600)),
                text(description)
                    .base_style(style().color(ColorToken::TextMuted).margin_bottom(12)),
                action,
            ))
            .base_style(
                style()
                    .padding_x(12)
                    .padding_top(24)
                    .padding_bottom(12)
                    .gap(6)
                    .align_items(Align::Center),
            )
            .into_view()
    }
}
