//! Role: Surface, data and feedback components of the Pop kit.

use crate::{Hue, Tokens};
use kivact::Value;
use kivact::control::{Node, NodeBuilder, column, icon, node, row, text};
use kivact::draw::{LineCap, LineJoin, Path, Point, Rect, Stroke};
use kivact::draw::{Shadow, Transform, linear_gradient};
use kivact::i18n::message;
use kivact::motion::{Animated, Easing, Transition, animated};
use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use kivact::style::{Align, Color, ColorToken, Direction, Justify, pct, style};
use kivact::view::{IntoText, IntoValue, IntoView, Role, View, show};

/// A bordered 14 px-radius surface with original 24 px content inset.
#[track_caller]
pub fn panel(content: impl IntoView) -> Node {
    column(content).style(move || {
        let t = &current_theme();
        let p = t.get::<Tokens>();
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
    hue: Hue,
    variant: BadgeVariant,
}
#[track_caller]
pub fn badge(label: impl IntoText) -> Badge {
    Badge {
        root: node(),
        label: label.into_text_value(),
        hue: Hue::Blue,
        variant: BadgeVariant::Soft,
    }
}
impl Badge {
    pub fn hue(mut self, hue: Hue) -> Self {
        self.hue = hue;
        self
    }
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }
}
impl NodeBuilder for Badge {
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
            let t = &current_theme();
            let p = t.get::<Tokens>();
            let (bg, fg) = p.hue(hue);
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
                        p.accent(hue),
                        match hue {
                            Hue::Yellow => kivact::style::rgb(0xffe98a),
                            Hue::Red => kivact::style::rgb(0xff9c6e),
                            Hue::Pink => kivact::style::rgb(0xff7cb0),
                            _ => p.cyan,
                        },
                    ]))
                    .color(if hue == Hue::Yellow {
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
                    .background(if hue == Hue::Pink { p.pink } else { bg })
                    .color(if hue == Hue::Pink { Color::WHITE } else { fg }),
                _ => s,
            }
        });
        if variant == BadgeVariant::Dot {
            root = root.child(node().style(move || {
                let t = &current_theme();
                style()
                    .size(6, 6)
                    .radius(3)
                    .background(t.get::<Tokens>().hue(hue).1)
            }));
        }
        root.child(label).into_view()
    }
}

pub struct Avatar {
    root: Node,
    initials: Value<String>,
    size: f32,
    hue: f64,
    halo: Value<bool>,
    online: Value<bool>,
}
#[track_caller]
pub fn avatar(initials: impl IntoText) -> Avatar {
    Avatar {
        root: node(),
        initials: initials.into_text_value(),
        size: 40.0,
        hue: 205.0,
        halo: false.into_value(),
        online: false.into_value(),
    }
}
impl Avatar {
    /// The diameter in logical pixels (at least 16; the default is 40). A
    /// pixel number rather than a length: the initials, the halo and the
    /// online dot are drawn in proportion to it.
    pub fn size(mut self, size: impl Into<f64>) -> Self {
        self.size = (size.into() as f32).max(16.0);
        self
    }
    pub fn hue(mut self, hue: f64) -> Self {
        self.hue = hue;
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
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Avatar {
    fn into_view(self) -> View {
        let size = self.size;
        let hue = self.hue;
        let root = self.root.style(
            style()
                .size(size, size)
                .shrink(0)
                .radius(999)
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .color(Color::WHITE)
                .background(linear_gradient(140).colors([
                    kivact::style::hsl(hue, 0.85, 0.7),
                    kivact::style::hsl(hue + 18.0, 0.75, 0.52),
                ]))
                .font_family("Barlow, sans-serif")
                .italic(true)
                .weight(800)
                .font_size(size * 0.36),
        );
        let halo = show(self.halo, move || {
            let offset = Animated::from(
                0.0f32,
                -3.0f32,
                Transition::tween(1600, Easing::EaseInOut)
                    .repeat(None)
                    .alternate(),
            );
            node().style(move || {
                style()
                    .position(kivact::style::Position::Absolute)
                    .left(pct(14))
                    .right(pct(14))
                    .top(size * -0.22)
                    .height(size * 0.2)
                    .radius(pct(50))
                    .border((size * 0.045).max(2.0), kivact::style::hsl(hue, 0.95, 0.62))
                    .shadow(Shadow::new(0, 0, 8, kivact::style::hsl(hue, 1.0, 0.65)))
                    .translate(0, offset.get())
            })
        });
        let online = show(self.online, move || {
            node().style(move || {
                let t = &current_theme();
                style()
                    .position(kivact::style::Position::Absolute)
                    .right(0)
                    .bottom(0)
                    .size(size * 0.28, size * 0.28)
                    .radius(999)
                    .background(t.get::<Tokens>().mint)
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
pub fn progress_ring(value: impl IntoValue<f64>, label: impl IntoText, hue: Hue) -> Node {
    let value = value.into_value();
    let label = label.into_text_value();
    let animated = animated(value, Transition::tween(1000, Easing::EaseOut));
    column((
        node()
            .role(Role::ProgressBar)
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
                let p = d.theme().get::<Tokens>();
                let centre = s.to_rect().center();
                let arc = |fraction: f32| {
                    let n = (fraction * 80.0).ceil().max(1.0) as usize;
                    let mut path = Path::new();
                    for i in 0..=n {
                        let angle = -std::f32::consts::FRAC_PI_2
                            + std::f32::consts::TAU * fraction * i as f32 / n as f32;
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
                    &arc((animated.get().clamp(0.0, 100.0) / 100.0) as f32),
                    &Stroke::new(6.0).cap(LineCap::Round).join(LineJoin::Round),
                    p.accent(hue),
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
) -> Node {
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
                let t = &current_theme();
                let p = t.get::<Tokens>();
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
                .role(Role::Heading)
                .level(2_u32)
                .style(style().font_size(20).weight(600).line_height(1.2)),
            text(subtitle).style(style().font_size(13).color(ColorToken::TextMuted)),
        )),
    ))
    .style(style().gap(16).align_items(Align::Center).margin_bottom(16))
}

#[track_caller]
pub fn sparkline(points: impl IntoValue<Vec<f32>>) -> Node {
    let points = points.into_value();
    node()
        .style(style().height(52).min_width(1))
        .draw(move |d, s| {
            let points = points.get();
            if points.len() < 2 {
                return;
            }
            let min = points.iter().copied().fold(f32::INFINITY, f32::min);
            let max = points.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let p = d.theme().get::<Tokens>();
            let mut line = Path::new();
            let mut area = kivact::draw::Path::new().move_to(Point::new(0.0, s.height));
            for (i, v) in points.iter().enumerate() {
                let pt = Point::new(
                    s.width * i as f32 / (points.len() - 1) as f32,
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
        trend_hue: Hue::Gray,
    }
}
/// Presentation builder returned by [`stat_card`].
pub struct StatCard {
    root: Node,
    title: Value<String>,
    value: Value<String>,
    trend: Value<String>,
    glyph: Option<Value<kivact::style::Icon>>,
    points: Value<Vec<f32>>,
    trend_hue: Hue,
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
    pub fn points(mut self, value: impl IntoValue<Vec<f32>>) -> Self {
        self.points = value.into_value();
        self
    }
    /// The trend hue; neutral gray by default.
    pub fn tone(mut self, value: Hue) -> Self {
        self.trend_hue = value;
        self
    }
}
impl NodeBuilder for StatCard {
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
                                    let t = &current_theme();
                                    let p = t.get::<Tokens>();
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
                                let t = &current_theme();
                                style().color(t.get::<Tokens>().hue(trend_hue).1)
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
                sparkline(points).base_style(style().margin_top(8)),
            ))
            .base_style(style().padding(0).min_height(185).clip(true))
            .into_view()
    }
}

#[track_caller]
pub fn key_value(key: impl IntoText, value: impl IntoView) -> Node {
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
        hue: Hue::Gray,
        trailing: None,
    }
}
/// Presentation builder returned by [`list_item`].
pub struct ListItem {
    root: Node,
    title: Value<String>,
    subtitle: Value<String>,
    glyph: Option<Value<kivact::style::Icon>>,
    hue: Hue,
    trailing: Option<crate::slot::Slot>,
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
    pub fn tone(mut self, value: Hue) -> Self {
        self.hue = value;
        self
    }
    /// Content built once when the row mounts.
    pub fn trailing<V: IntoView>(mut self, value: impl FnOnce() -> V + 'static) -> Self {
        self.trailing = Some(crate::slot::slot(value));
        self
    }
}
impl NodeBuilder for ListItem {
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
                            let t = &current_theme();
                            let p = t.get::<Tokens>();
                            let (bg, fg) = p.hue(hue);
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
                trailing.map(|slot| slot()),
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
        root: row(()).role(Role::Alert),
        actions: None,
        title: title.into_text_value(),
        body: String::new().into_value(),
        tone: kivact::ui::Tone::Neutral,
    }
}
/// Presentation builder returned by [`alert`].
pub struct Alert {
    root: Node,
    actions: Option<crate::slot::Slot>,
    title: Value<String>,
    body: Value<String>,
    tone: kivact::ui::Tone,
}
impl Alert {
    /// Optional actions built once when the alert mounts.
    pub fn actions<V: IntoView>(mut self, actions: impl FnOnce() -> V + 'static) -> Self {
        self.actions = Some(crate::slot::slot(actions));
        self
    }

    /// Supporting text; empty by default.
    pub fn description(mut self, value: impl IntoText) -> Self {
        self.body = value.into_text_value();
        self
    }
    /// Semantic accent; neutral by default.
    pub fn tone(mut self, tone: kivact::ui::Tone) -> Self {
        self.tone = tone;
        self
    }
}
impl NodeBuilder for Alert {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Alert {
    fn into_view(self) -> View {
        let title = self.title;
        let body = self.body;
        let tone = self.tone;

        use kivact::ui::Tone;
        let hue = match tone {
            Tone::Neutral => Hue::Gray,
            Tone::Info => Hue::Blue,
            Tone::Success => Hue::Mint,
            Tone::Warning => Hue::Yellow,
            Tone::Danger => Hue::Red,
            Tone::Accent => Hue::Pink,
            // Variants added later fall back to the neutral tone.
            _ => Hue::Gray,
        };
        self.root
            .child((
                icon(if hue == Hue::Red {
                    "alert-circle"
                } else if hue == Hue::Yellow {
                    "alert-triangle"
                } else if hue == Hue::Mint {
                    "circle-check"
                } else {
                    "info-circle"
                })
                .base_style(move || {
                    let t = &current_theme();
                    style().color(t.get::<Tokens>().hue(hue).1)
                })
                .base_style(style().margin_top(2)),
                column((
                    text(title)
                        .base_style(style().font_size(14).weight(500).line_height(1.5))
                        .base_style(move || {
                            let t = &current_theme();
                            let p = t.get::<Tokens>();
                            style().color(if hue == Hue::Red {
                                p.hue(hue).1
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
                let t = &current_theme();
                let p = t.get::<Tokens>();
                style()
                    .align_items(if hue == Hue::Red {
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
                        if hue == Hue::Red {
                            p.hue(hue).1.mix_in(p.line, 0.7, t)
                        } else {
                            p.line
                        },
                    )
            })
            .child(self.actions.map(|slot| slot()))
            .into_view()
    }
}

#[track_caller]
pub fn spinner() -> Node {
    let angle = Animated::from(
        0.0f32,
        360.0f32,
        Transition::tween(700, Easing::Linear)
            .repeat(None)
            .decorative(false),
    );
    node()
        .role(Role::Status)
        .name(message("pop.loader.label", "Loading"))
        .style(move || style().size(20, 20).radius(999).rotate(angle.get()))
        .draw(|draw, size| {
            let p = draw.theme().get::<Tokens>();
            let thickness = if size.width.min(size.height) >= 28.0 {
                3.0
            } else {
                2.0
            };
            let centre = size.to_rect().center();
            let radius = (size.width.min(size.height) - thickness) / 2.0;
            let arc = |fraction: f32| {
                let mut path = Path::new();
                for i in 0..=64 {
                    let angle = -std::f32::consts::FRAC_PI_2
                        + std::f32::consts::TAU * fraction * i as f32 / 64.0;
                    let point = Point::new(
                        centre.x + radius * angle.cos(),
                        centre.y + radius * angle.sin(),
                    );
                    path = if i == 0 {
                        path.move_to(point)
                    } else {
                        path.line_to(point)
                    };
                }
                path
            };
            draw.stroke_path(
                &arc(1.0),
                &Stroke::new(thickness)
                    .cap(LineCap::Round)
                    .join(LineJoin::Round),
                p.surface_active,
            );
            draw.stroke_path(
                &arc(0.25),
                &Stroke::new(thickness)
                    .cap(LineCap::Round)
                    .join(LineJoin::Round),
                p.blue,
            );
        })
}
/// The three-color bouncing dots use the same public motion clock as the host.
#[track_caller]
pub fn dots_loader() -> Node {
    let phase = Animated::from(
        0.0f32,
        std::f32::consts::TAU,
        Transition::tween(1000, Easing::Linear)
            .repeat(None)
            .decorative(false),
    );
    node()
        .role(Role::Status)
        .name(message("pop.loader.label", "Loading"))
        .style(style().size(34, 20))
        .draw(move |draw, _| {
            let p = draw.theme().get::<Tokens>();
            for (i, color) in [p.blue, p.cyan, p.yellow].into_iter().enumerate() {
                let bounce = (phase.get() - i as f32 * 0.95).sin().max(0.0) * 6.0;
                draw.rounded_rect(
                    Rect::new(i as f32 * 13.0, 10.0 - bounce, 8.0, 8.0),
                    4.0,
                    color,
                );
            }
        })
}
#[track_caller]
pub fn skeleton() -> Skeleton {
    Skeleton {
        root: kivact::control::column(()),
        rows: 1usize.into_value(),
    }
}
/// Presentation builder returned by [`skeleton`].
pub struct Skeleton {
    root: Node,
    rows: kivact::Value<usize>,
}
impl Skeleton {
    /// Reactive row count; one by default.
    pub fn rows(mut self, rows: impl IntoValue<usize>) -> Self {
        self.rows = rows.into_value();
        self
    }
}
impl NodeBuilder for Skeleton {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Skeleton {
    fn into_view(self) -> View {
        let rows = self.rows;
        let pulse = Animated::from(
            1.0f32,
            0.5f32,
            Transition::tween(1000, Easing::EaseInOut)
                .repeat(None)
                .alternate(),
        );
        self.root
            .base_style(style().gap(6).clip(true))
            .child(kivact::view::for_index(
                move || (0..rows.get()).collect::<Vec<_>>(),
                move |_, _| {
                    node().style(move || {
                        style()
                            .grow(1)
                            .min_height(16)
                            .width(pct(100))
                            .radius(6)
                            .background(ColorToken::SurfaceAlt)
                            .opacity(if kivact::platform::reduced_motion() {
                                1.
                            } else {
                                pulse.get()
                            })
                    })
                },
            ))
            .into_view()
    }
}

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
    action: Option<crate::slot::Slot>,
}
impl EmptyState {
    /// Supporting text; empty by default.
    pub fn description(mut self, value: impl IntoText) -> Self {
        self.description = value.into_text_value();
        self
    }
    /// Optional content built once when the state mounts.
    pub fn content<V: IntoView>(mut self, value: impl FnOnce() -> V + 'static) -> Self {
        self.action = Some(crate::slot::slot(value));
        self
    }
}
impl NodeBuilder for EmptyState {
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
                    let p = d.theme().get::<Tokens>();
                    let mut halo = Path::new();
                    for i in 0..=64 {
                        let a = std::f32::consts::TAU * i as f32 / 64.0;
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
                    d.rounded_rect(Rect::new(38.0, 42.0, 84.0, 56.0), 12.0, p.hue(Hue::Blue).0);
                    d.line(
                        Point::new(38.0, 60.0),
                        Point::new(122.0, 60.0),
                        2.0,
                        p.line_strong,
                    );
                    for x in [66.0, 94.0] {
                        d.rounded_rect(Rect::new(x - 3.5, 74.5, 7.0, 7.0), 3.5, p.text_faint);
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
                action.map(|slot| slot()),
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
