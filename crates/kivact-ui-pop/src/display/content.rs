//! Role: Render Pop surfaces, identity badges and data displays.

use crate::{Hue, Tokens};
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::draw::{Shadow, Transform, linear_gradient};
use kivact::geometry::Point;
use kivact::motion::{Animated, Easing, Repeat, Transition, animated};
use kivact::state::Value;
use kivact::state::{IntoText, IntoValue};
use kivact::style::use_theme;
use kivact::style::{Align, Color, ColorToken, Direction, Justify, pct, style};
use kivact::view::{IntoView, Role, View, show};
use kivact::view::{Node, NodeBuilder, column, icon, node, row, text};

/// A bordered 14 px-radius surface with original 24 px content inset.
#[track_caller]
pub fn panel(content: impl IntoView) -> Node {
    column(content).style(move || {
        let t = &use_theme().get();
        let p = t.ext::<Tokens>();
        style()
            .padding(p.panel_padding)
            .background(p.surface)
            .border(1, p.line)
            .radius(p.panel_radius)
            .shadow(Shadow::new(0, 1, 3, ColorToken::Shadow))
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Soft,
    Dot,
    Solid,
    Slant,
    Count,
}
pub struct Badge {
    root: Node,
    label: Value<String>,
    hue: Value<Hue>,
    variant: BadgeVariant,
}
#[track_caller]
pub fn badge(label: impl IntoText) -> Badge {
    Badge {
        root: node(),
        label: label.into_text_value(),
        hue: Hue::Blue.into_value(),
        variant: BadgeVariant::Soft,
    }
}
impl Badge {
    pub fn hue(mut self, hue: impl IntoValue<Hue>) -> Self {
        self.hue = hue.into_value();
        self
    }
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }
}
impl NodeBuilder for Badge {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Badge {
    fn into_view(self) -> View {
        let hue = self.hue;
        let variant = self.variant;
        let mut label = text(self.label);
        if variant == BadgeVariant::Slant {
            label = label.style(style().transform(Transform::new().skew_x(16)));
        }
        let mut root = self.root.base_style(move || {
            let t = &use_theme().get();
            let p = t.ext::<Tokens>();
            let (bg, fg) = p.hue(hue.get());
            let s = style()
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .gap(4)
                .height(22)
                .shrink(0)
                .padding_x(8)
                .radius(6)
                .font_size(12)
                .weight(500)
                .background(bg)
                .color(fg)
                .border(1, Color::TRANSPARENT)
                .text_wrap(false);
            match variant {
                BadgeVariant::Dot => s
                    .background(Color::TRANSPARENT)
                    .color(p.text)
                    .border_color(p.line),
                BadgeVariant::Solid => s.background(p.text).color(p.surface).weight(600),
                BadgeVariant::Slant => s
                    .background(linear_gradient(90).colors([
                        p.accent(hue.get()),
                        match hue.get() {
                            Hue::Yellow => kivact::style::rgb(0xffe98a),
                            Hue::Red => kivact::style::rgb(0xff9c6e),
                            Hue::Pink => kivact::style::rgb(0xff7cb0),
                            _ => p.cyan,
                        },
                    ]))
                    .color(if hue.get() == Hue::Yellow {
                        kivact::style::rgb(0x3b2f00)
                    } else {
                        Color::WHITE
                    })
                    .radius(4)
                    .padding_x(12)
                    .font_family("Barlow, sans-serif")
                    .italic(true)
                    .weight(800)
                    .transform(Transform::new().skew_x(-16)),
                BadgeVariant::Count => s
                    .height(20)
                    .min_width(20)
                    .padding_x(6)
                    .radius(999)
                    .background(if hue.get() == Hue::Pink { p.pink } else { bg })
                    .color(if hue.get() == Hue::Pink {
                        Color::WHITE
                    } else {
                        fg
                    }),
                _ => s,
            }
        });
        if variant == BadgeVariant::Dot {
            root = root.child(node().style(move || {
                let t = &use_theme().get();
                style()
                    .size(6, 6)
                    .radius(3)
                    .background(t.ext::<Tokens>().hue(hue.get()).1)
            }));
        }
        root.child(label).into_view()
    }
}

pub struct Avatar {
    root: Node,
    initials: Value<String>,
    size: Value<f64>,
    hue: Value<f64>,
    halo: Value<bool>,
    online: Value<bool>,
}
#[track_caller]
pub fn avatar(initials: impl IntoText) -> Avatar {
    Avatar {
        root: node(),
        initials: initials.into_text_value(),
        size: 40.0.into_value(),
        hue: 205.0.into_value(),
        halo: false.into_value(),
        online: false.into_value(),
    }
}
impl Avatar {
    /// The diameter in logical pixels (at least 16; the default is 40). A
    /// pixel number rather than a length: the initials, the halo and the
    /// online dot are drawn in proportion to it.
    pub fn size(mut self, size: impl IntoValue<f64>) -> Self {
        let size = size.into_value();
        self.size = (move || size.get().max(16.0)).into_value();
        self
    }
    pub fn hue(mut self, hue: impl IntoValue<f64>) -> Self {
        let hue = hue.into_value();
        self.hue = hue.into_value();
        self
    }
    /// Draw an animated halo over the avatar while true (constant or
    /// reactive).
    #[track_caller]
    pub fn halo(mut self, on: impl IntoValue<bool>) -> Self {
        self.halo = on.into_value();
        self
    }
    /// Draw the online dot while true (constant or reactive).
    #[track_caller]
    pub fn online(mut self, on: impl IntoValue<bool>) -> Self {
        self.online = on.into_value();
        self
    }
}
impl NodeBuilder for Avatar {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Avatar {
    fn into_view(self) -> View {
        let size = self.size;
        let hue = self.hue;
        let root = self.root.style(move || {
            style()
                .size(size.get(), size.get())
                .shrink(0)
                .radius(999)
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .color(Color::WHITE)
                .background(linear_gradient(140).colors([
                    kivact::style::hsl(hue.get(), 0.85, 0.7),
                    kivact::style::hsl(hue.get() + 18.0, 0.75, 0.52),
                ]))
                .font_family("Barlow, sans-serif")
                .italic(true)
                .weight(800)
                .font_size(size.get() * 0.36)
        });
        let halo = show(self.halo, move || {
            let offset = Animated::from(
                0.0f64,
                -3.0f64,
                Transition::tween(1600, Easing::EaseInOut)
                    .repeat(Repeat::Forever)
                    .alternate(),
            );
            node().style(move || {
                style()
                    .position(kivact::style::Position::Absolute)
                    .left(pct(14))
                    .right(pct(14))
                    .top(size.get() * -0.22)
                    .height(size.get() * 0.2)
                    .radius(pct(50))
                    .border(
                        (size.get() * 0.045).max(2.0),
                        kivact::style::hsl(hue.get(), 0.95, 0.62),
                    )
                    .shadow(Shadow::new(
                        0,
                        0,
                        8,
                        kivact::style::hsl(hue.get(), 1.0, 0.65),
                    ))
                    .translate(0, offset.get())
            })
        });
        let online = show(self.online, move || {
            node().style(move || {
                let t = &use_theme().get();
                style()
                    .position(kivact::style::Position::Absolute)
                    .right(0)
                    .bottom(0)
                    .size(size.get() * 0.28, size.get() * 0.28)
                    .radius(999)
                    .background(t.ext::<Tokens>().mint)
                    .ring(2, t.colors().surface)
            })
        });
        root.child(halo)
            .child(online)
            .child(text(self.initials))
            .into_view()
    }
}

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
                    .style(style().background(crate::Color::Yellow)),
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
        root: panel(()).style(style().padding(0)),
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
#[track_caller]
pub fn list_item(title: impl IntoText) -> ListItem {
    ListItem {
        root: row(()),
        title: title.into_text_value(),
        subtitle: String::new().into_value(),
        glyph: None,
        hue: Hue::Gray.into_value(),
        trailing: None,
    }
}
/// Presentation builder returned by [`list_item`].
pub struct ListItem {
    root: Node,
    title: Value<String>,
    subtitle: Value<String>,
    glyph: Option<Value<kivact::style::Icon>>,
    hue: Value<Hue>,
    trailing: Option<kivact::view::View>,
}
impl ListItem {
    /// Optional subtitle; empty by default.
    pub fn subtitle(mut self, value: impl IntoText) -> Self {
        self.subtitle = value.into_text_value();
        self
    }
    /// Optional reactive icon.
    pub fn icon(mut self, value: impl IntoValue<kivact::style::Icon>) -> Self {
        self.glyph = Some(value.into_value());
        self
    }
    /// Icon hue; neutral gray by default.
    pub fn tone(mut self, value: impl IntoValue<Hue>) -> Self {
        self.hue = value.into_value();
        self
    }
    /// Content built once when the row mounts.
    pub fn trailing(mut self, value: impl IntoView) -> Self {
        self.trailing = Some(value.into_view());
        self
    }
}
impl NodeBuilder for ListItem {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for ListItem {
    fn into_view(self) -> View {
        let title = self.title;
        let subtitle = self.subtitle;
        let glyph = self.glyph;
        let hue = self.hue;
        let trailing = self.trailing;

        self.root
            .child((
                glyph.map(|glyph| {
                    node()
                        .base_style(move || {
                            let t = &use_theme().get();
                            let p = t.ext::<Tokens>();
                            let (bg, fg) = p.hue(hue.get());
                            style()
                                .size(36, 36)
                                .radius(8)
                                .background(bg)
                                .color(fg)
                                .direction(Direction::Row)
                                .align_items(Align::Center)
                                .justify_content(Justify::Center)
                                .shrink(0)
                        })
                        .child(icon(glyph))
                }),
                column((
                    text(title).base_style(style().weight(500)),
                    text(subtitle).base_style(style().font_size(12).color(ColorToken::TextMuted)),
                ))
                .base_style(style().grow(1)),
                trailing,
            ))
            .base_style(
                style()
                    .padding_x(8)
                    .padding_y(10)
                    .gap(12)
                    .align_items(Align::Center)
                    .radius(8)
                    .hover(style().background(ColorToken::SurfaceAlt)),
            )
            .into_view()
    }
}

#[track_caller]
pub fn alert(title: impl IntoText) -> Alert {
    Alert {
        root: row(()).with_node(|root| root.role(Role::Alert)),
        actions: None,
        title: title.into_text_value(),
        body: String::new().into_value(),
        tone: kivact::interaction::Tone::Neutral.into_value(),
    }
}
/// Presentation builder returned by [`alert`].
pub struct Alert {
    root: Node,
    actions: Option<kivact::view::View>,
    title: Value<String>,
    body: Value<String>,
    tone: Value<kivact::interaction::Tone>,
}
impl Alert {
    /// Optional actions built once when the alert mounts.
    pub fn actions(mut self, actions: impl IntoView) -> Self {
        self.actions = Some(actions.into_view());
        self
    }

    /// Supporting text; empty by default.
    pub fn description(mut self, value: impl IntoText) -> Self {
        self.body = value.into_text_value();
        self
    }
    /// Semantic accent; neutral by default.
    pub fn tone(mut self, tone: impl IntoValue<kivact::interaction::Tone>) -> Self {
        self.tone = tone.into_value();
        self
    }
}
impl NodeBuilder for Alert {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Alert {
    fn into_view(self) -> View {
        let title = self.title;
        let body = self.body;
        let tone = self.tone;

        use kivact::interaction::Tone;
        let hue = kivact::state::memo(move || match tone.get() {
            Tone::Neutral => Hue::Gray,
            Tone::Info => Hue::Blue,
            Tone::Success => Hue::Mint,
            Tone::Warning => Hue::Yellow,
            Tone::Danger => Hue::Red,
            Tone::Accent => Hue::Pink,
        });
        self.root
            .child((
                icon(move || {
                    kivact::style::Icon::named(if hue.get() == Hue::Red {
                        "alert-circle"
                    } else if hue.get() == Hue::Yellow {
                        "alert-triangle"
                    } else if hue.get() == Hue::Mint {
                        "circle-check"
                    } else {
                        "info-circle"
                    })
                })
                .base_style(move || {
                    let t = &use_theme().get();
                    style().color(t.ext::<Tokens>().hue(hue.get()).1)
                })
                .base_style(style().margin_top(2)),
                column((
                    text(title)
                        .base_style(style().font_size(14).weight(500).line_height(1.5))
                        .base_style(move || {
                            let t = &use_theme().get();
                            let p = t.ext::<Tokens>();
                            style().color(if hue.get() == Hue::Red {
                                p.hue(hue.get()).1
                            } else {
                                p.text
                            })
                        }),
                    text(body).base_style(
                        style()
                            .font_size(13)
                            .line_height(1.5)
                            .color(ColorToken::TextMuted),
                    ),
                ))
                .base_style(style().gap(2).grow(1)),
            ))
            .base_style(move || {
                let t = &use_theme().get();
                let p = t.ext::<Tokens>();
                style()
                    .align_items(if hue.get() == Hue::Red {
                        Align::Center
                    } else {
                        Align::Start
                    })
                    .padding_x(16)
                    .padding_y(12)
                    .gap(12)
                    .radius(10)
                    .background(p.surface)
                    .border(
                        1,
                        if hue.get() == Hue::Red {
                            p.hue(hue.get()).1.mix_in(p.line, 0.7, t)
                        } else {
                            p.line
                        },
                    )
            })
            .child(self.actions)
            .into_view()
    }
}
