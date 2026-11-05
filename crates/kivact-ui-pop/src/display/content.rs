//! Role: Render Pop surfaces, identity badges and data displays.

use crate::{Hue, Tokens};
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::draw::{Shadow, Transform, linear_gradient};
use kivact::geometry::Point;
use kivact::motion::{Easing, Transition, animated};
use kivact::state::Value;
use kivact::state::{IntoText, IntoValue};
use kivact::style::use_theme;
use kivact::style::{Align, Color, ColorToken, Direction, Justify, pct, style};
use kivact::view::{IntoView, Role, View};
use kivact::view::{Node, NodeBuilder, column, icon, node, row, text};

/// A 72 px circular progress gauge, with the percentage retained as text.
#[track_caller]
pub fn progress_ring(
    value: impl IntoValue<f64>,
    label: impl IntoText,
    hue: Value<Hue>,
) -> impl kivact::view::Component {
    let value = value.into_value();
    let label = label.into_text_value();
    let animated = animated(value, Transition::tween(1000, Easing::EaseOut));
    column((
        node()
            .with_node(|root| root.role(Role::ProgressBar))
            .name(label.into_value())
            .value_min(0.0)
            .value_max(100.0)
            .value_now(value)
            .style(
                style()
                    .size(72, 72)
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center),
            )
            .draw(move |d, s| {
                let p = d.theme().ext::<Tokens>();
                let centre = s.to_rect().center();
                let arc = |fraction: f64| {
                    let n = (fraction * 80.0).ceil().max(1.0) as usize;
                    let mut path = Path::new();
                    for i in 0..=n {
                        let angle = -std::f64::consts::FRAC_PI_2
                            + std::f64::consts::TAU * fraction * i as f64 / n as f64;
                        let pt = Point::new(
                            centre.x + 27.0 * angle.cos(),
                            centre.y + 27.0 * angle.sin(),
                        );
                        path = if i == 0 {
                            path.move_to(pt)
                        } else {
                            path.line_to(pt)
                        };
                    }
                    path
                };
                d.stroke_path(
                    &arc(1.0),
                    &Stroke::new(6.0).cap(LineCap::Round).join(LineJoin::Round),
                    p.surface_active,
                );
                d.stroke_path(
                    &arc((animated.get().clamp(0.0, 100.0) / 100.0) as f64),
                    &Stroke::new(6.0).cap(LineCap::Round).join(LineJoin::Round),
                    p.accent(hue.get()),
                );
            })
            .child(
                text(move || format!("{:.0}%", value.get()))
                    .style(style().font_size(16).weight(600)),
            ),
        text(label).style(style().font_size(12).color(ColorToken::TextMuted)),
    ))
    .style(style().align_items(Align::Center).gap(6))
}

#[track_caller]
pub fn section_heading(
    number: impl Into<String>,
    title: impl IntoText,
    subtitle: impl IntoText,
) -> impl kivact::view::Component {
    let number = number.into();
    row((
        node()
            .style(
                style()
                    .size(44, 30)
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .radius(6)
                    .transform(Transform::new().skew_x(-14))
                    .color(Color::WHITE),
            )
            .style(move || {
                let t = &use_theme().get();
                let p = t.ext::<Tokens>();
                style().background(linear_gradient(90).colors([p.blue, p.cyan]))
            })
            .child(
                text(number).style(
                    style()
                        .font_family("Barlow, sans-serif")
                        .italic(true)
                        .weight(800)
                        .font_size(16)
                        .color(Color::WHITE)
                        .transform(Transform::new().skew_x(14)),
                ),
            )
            .child(
                node()
                    .style(
                        style()
                            .position(kivact::style::Position::Absolute)
                            .right(-9)
                            .top(4)
                            .size(5, 22)
                            .radius(2),
                    )
                    .style(style().background(crate::ColorName::Yellow)),
            ),
        column((
            text(title)
                .with_node(|root| root.role(Role::Heading))
                .level(2_u32)
                .style(style().font_size(20).weight(600).line_height(1.2)),
            text(subtitle).style(style().font_size(13).color(ColorToken::TextMuted)),
        )),
    ))
    .style(style().gap(16).align_items(Align::Center).margin_bottom(16))
}

#[track_caller]
pub fn sparkline(points: impl IntoValue<Vec<f64>>) -> impl kivact::view::Component {
    let points = points.into_value();
    node()
        .style(style().height(52).min_width(1))
        .draw(move |d, s| {
            let points = points.get();
            if points.len() < 2 {
                return;
            }
            let min = points.iter().copied().fold(f64::INFINITY, f64::min);
            let max = points.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let p = d.theme().ext::<Tokens>();
            let mut line = Path::new();
            let mut area = kivact::draw::Path::new().move_to(Point::new(0.0, s.height));
            for (i, v) in points.iter().enumerate() {
                let pt = Point::new(
                    s.width * i as f64 / (points.len() - 1) as f64,
                    6.0 + (1.0 - (*v - min) / (max - min).max(1.0)) * (s.height - 12.0),
                );
                line = if i == 0 {
                    line.move_to(pt)
                } else {
                    line.line_to(pt)
                };
                area = area.line_to(pt);
            }
            d.fill_path(
                &area.line_to(Point::new(s.width, s.height)).close(),
                linear_gradient(180)
                    .stop(kivact::style::rgba(0x2b8bd8, 0.32), pct(0))
                    .stop(Color::TRANSPARENT, pct(100)),
            );
            d.stroke_path(
                &line,
                &Stroke::new(2.4).cap(LineCap::Round).join(LineJoin::Round),
                p.blue,
            );
        })
}

/// A statistic with application-provided trend text, points and semantic hue.
#[track_caller]
pub fn stat_card(title: impl IntoText, value: impl IntoText) -> StatCard {
    StatCard {
        root: column(()).style(move || {
            let t = use_theme().get();
            let p = t.ext::<Tokens>();
            style()
                .background(p.surface)
                .border(1, p.line)
                .radius(p.panel_radius)
                .shadow(Shadow::new(0, 1, 3, ColorToken::Shadow))
        }),
        title: title.into_text_value(),
        value: value.into_text_value(),
        trend: String::new().into_value(),
        glyph: None,
        points: Vec::new().into_value(),
        trend_hue: Hue::Gray.into_value(),
    }
}
/// Presentation builder returned by [`stat_card`].
pub struct StatCard {
    root: Node,
    title: Value<String>,
    value: Value<String>,
    trend: Value<String>,
    glyph: Option<Value<kivact::style::Icon>>,
    points: Value<Vec<f64>>,
    trend_hue: Value<Hue>,
}
impl StatCard {
    /// Optional trend text; empty by default.
    pub fn trend(mut self, value: impl IntoText) -> Self {
        self.trend = value.into_text_value();
        self
    }
    /// Optional reactive icon.
    pub fn icon(mut self, value: impl IntoValue<kivact::style::Icon>) -> Self {
        self.glyph = Some(value.into_value());
        self
    }
    /// Read-only sparkline samples; empty by default.
    pub fn points(mut self, value: impl IntoValue<Vec<f64>>) -> Self {
        self.points = value.into_value();
        self
    }
    /// The trend hue; neutral gray by default.
    pub fn tone(mut self, value: impl IntoValue<Hue>) -> Self {
        self.trend_hue = value.into_value();
        self
    }
}
impl NodeBuilder for StatCard {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for StatCard {
    fn into_view(self) -> View {
        let title = self.title;
        let value = self.value;
        let trend = self.trend;
        let glyph = self.glyph;
        let points = self.points;
        let trend_hue = self.trend_hue;

        self.root
            .child((
                column((
                    row((
                        glyph.map(|glyph| {
                            node()
                                .base_style(move || {
                                    let t = &use_theme().get();
                                    let p = t.ext::<Tokens>();
                                    style()
                                        .size(32, 32)
                                        .radius(8)
                                        .background(p.hue(Hue::Blue).0)
                                        .color(p.text_muted)
                                        .direction(Direction::Row)
                                        .align_items(Align::Center)
                                        .justify_content(Justify::Center)
                                })
                                .child(icon(glyph))
                        }),
                        text(trend)
                            .base_style(move || {
                                let t = &use_theme().get();
                                style().color(t.ext::<Tokens>().hue(trend_hue.get()).1)
                            })
                            .base_style(
                                style()
                                    .font_size(11)
                                    .padding_x(6)
                                    .padding_y(1)
                                    .radius(4)
                                    .border(1, ColorToken::Border)
                                    .background(ColorToken::Surface),
                            ),
                    ))
                    .base_style(
                        style()
                            .justify_content(Justify::SpaceBetween)
                            .align_items(Align::Center)
                            .margin_bottom(8),
                    ),
                    text(title).base_style(
                        style()
                            .font_size(13)
                            .weight(500)
                            .color(ColorToken::TextMuted),
                    ),
                    text(value).base_style(style().font_size(26).weight(600).line_height(1.2)),
                ))
                .base_style(style().padding(20).padding_bottom(0).gap(4)),
                sparkline(points).with_node(|node| node.base_style(style().margin_top(8))),
            ))
            .base_style(style().padding(0).min_height(185).clip(true))
            .into_view()
    }
}

#[track_caller]
pub fn key_value(key: impl IntoText, value: impl IntoView) -> impl kivact::view::Component {
    row((
        text(key).style(
            style()
                .width(110)
                .font_size(13)
                .color(ColorToken::TextMuted),
        ),
        row(value).style(style().grow(1).align_items(Align::Center)),
    ))
    .style(
        style()
            .gap(12)
            .align_items(Align::Center)
            .min_height(44)
            .border_bottom(1, ColorToken::Border),
    )
}
