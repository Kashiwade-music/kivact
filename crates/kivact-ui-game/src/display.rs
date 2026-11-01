//! Role: Dense data displays with hard-edged status geometry.
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::draw::{Path, Point, Rect};
use kivact::draw::{Shape, Stroke, Transform};
use kivact::motion::{Animated, Easing, Transition};
use kivact::platform;
use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use kivact::style::{Align, Color, ColorToken, Justify, Theme, pct, rgba, style};
use kivact::view::{IntoText, IntoValue, IntoView, NodeRef, Role, View, node_ref};
use kivact::{Signal, Value, effect, signal};

use crate::Tokens;
use kivact::ui::Tone;

/// The colors a game theme gives a semantic [`Tone`].
pub(crate) trait ToneExt {
    /// The tone's solid color.
    fn color(self, t: &Theme) -> Color;
    /// The tone's weak tint, for backgrounds.
    fn weak(self, t: &Theme) -> Color;
}

impl ToneExt for Tone {
    fn color(self, t: &Theme) -> Color {
        match self {
            Self::Info => t.colors().accent,
            Self::Success => t.colors().success,
            Self::Warning => t.colors().warning,
            Self::Danger => t.colors().danger,
            Self::Neutral => t.get::<Tokens>().text_2,
            Self::Accent => t.get::<Tokens>().yellow,
        }
    }
    fn weak(self, t: &Theme) -> Color {
        let dark = t.get::<Tokens>().mode == kivact::style::ThemeMode::Dark;
        match self {
            Self::Info => t.get::<Tokens>().primary_weak,
            Self::Success => {
                if dark {
                    rgba(0x2cc48a, 0.14)
                } else {
                    kivact::style::rgb(0xddf5eb)
                }
            }
            Self::Warning => {
                if dark {
                    rgba(0xf5a524, 0.14)
                } else {
                    kivact::style::rgb(0xfff1d6)
                }
            }
            Self::Danger => {
                if dark {
                    rgba(0xff5a6e, 0.14)
                } else {
                    kivact::style::rgb(0xfde4e8)
                }
            }
            Self::Neutral => t.get::<Tokens>().surface_3,
            Self::Accent => t.get::<Tokens>().yellow,
        }
    }
}

/// Skew-ended status badge with a square status dot.
#[track_caller]
pub fn badge(label: impl IntoText, tone: Tone) -> Node {
    tag_badge(label, tone, BadgeVariant::Dot)
}

/// Status, version and environment badge treatments.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Dot,
    Plain,
    Solid,
    Outline,
}

/// A prototype badge with an explicit treatment, retaining generic modifiers.
#[track_caller]
pub fn tag_badge(label: impl IntoText, tone: Tone, variant: BadgeVariant) -> Node {
    let dot = if variant == BadgeVariant::Dot {
        Some(node().style(move || {
            let t = &current_theme();
            style().size(6, 6).background(tone.color(t))
        }))
    } else {
        None
    };
    row((
        dot,
        crate::mono(label).style(move || {
            let t = &current_theme();
            style().color(if tone == Tone::Accent {
                t.get::<Tokens>().navy
            } else if variant == BadgeVariant::Solid {
                t.colors().accent_text
            } else {
                tone.color(t)
            })
        }),
    ))
    .style(move || {
        let t = &current_theme();
        let mut s = style()
            .height(20)
            .align_self(Align::Start)
            .padding_x(9)
            .gap(5)
            .align_items(Align::Center)
            .color(if tone == Tone::Accent {
                t.get::<Tokens>().navy
            } else if variant == BadgeVariant::Solid {
                t.colors().accent_text
            } else {
                tone.color(t)
            })
            .background(match variant {
                BadgeVariant::Solid => tone.color(t),
                BadgeVariant::Outline => Color::TRANSPARENT,
                _ => tone.weak(t),
            })
            .clip_path(
                Shape::polygon()
                    .point(4, 0)
                    .point(pct(100), 0)
                    .point(pct(100) - 4, pct(100))
                    .point(0, pct(100)),
            );
        if variant == BadgeVariant::Outline {
            s = s
                .clip_path(
                    Shape::polygon()
                        .point(0, 0)
                        .point(pct(100), 0)
                        .point(pct(100), pct(100))
                        .point(0, pct(100)),
                )
                .border(1, tone.color(t));
        }
        s
    })
}

/// KPI tile with reveal count-up, change text and a small trend line.
///
/// `label` is the small monospaced heading; [`Kpi::heading`] replaces it with
/// an application-owned view, such as an icon and text label.
#[track_caller]
pub fn kpi(label: impl IntoText, target: f64) -> Kpi {
    Kpi {
        root: column(()),
        label: label.into_text_value(),
        heading: None,
        target,
        unit: String::new().into_value(),
        change: String::new().into_value(),
        tone: Tone::Neutral,
        trend: Vec::new().into_value(),
        formatter: std::rc::Rc::new(|value| format!("{value:.0}")),
    }
}

/// KPI presentation builder. Applications supply trend samples and optional formatting.
pub struct Kpi {
    root: Node,
    label: Value<String>,
    heading: Option<crate::slot::Slot>,
    target: f64,
    unit: Value<String>,
    change: Value<String>,
    tone: Tone,
    trend: Value<Vec<f64>>,
    formatter: std::rc::Rc<dyn Fn(f64) -> String>,
}
impl Kpi {
    /// Decimal places in the default numeric format; zero by default.
    pub fn decimals(mut self, decimals: usize) -> Self {
        self.formatter = std::rc::Rc::new(move |value| format!("{value:.decimals$}"));
        self
    }
    /// Unit suffix; empty by default.
    pub fn unit(mut self, value: impl IntoText) -> Self {
        self.unit = value.into_text_value();
        self
    }
    /// Change caption; empty by default.
    pub fn change(mut self, value: impl IntoText) -> Self {
        self.change = value.into_text_value();
        self
    }
    /// Semantic accent; neutral by default.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// An application-owned heading in place of the label text, built once
    /// by `heading` when the tile mounts.
    pub fn heading<V: IntoView>(mut self, heading: impl FnOnce() -> V + 'static) -> Self {
        self.heading = Some(crate::slot::slot(heading));
        self
    }
    /// Read-only trend samples; the default empty trend draws no sparkline.
    pub fn trend(mut self, trend: impl IntoValue<Vec<f64>>) -> Self {
        self.trend = trend.into_value();
        self
    }
    /// Application-defined numeric text. The default uses ungrouped decimal digits.
    pub fn format(mut self, format: impl Fn(f64) -> String + 'static) -> Self {
        self.formatter = std::rc::Rc::new(format);
        self
    }
}
impl NodeBuilder for Kpi {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Kpi {
    fn into_view(mut self) -> View {
        let tone = self.tone;
        self.root = self.root.base_style(move || {
            let t = &current_theme();
            style()
                .padding_x(16)
                .padding_top(14)
                .padding_bottom(12)
                .gap(6)
                .background(t.colors().surface)
                .border(1, t.colors().border)
                .border_top(3, tone.color(t))
        });
        let heading = match self.heading {
            Some(heading) => heading(),
            None => crate::mono(self.label)
                .style(style().color(ColorToken::TextMuted))
                .into_view(),
        };
        let (target, unit, change, tone, formatter) = (
            self.target,
            self.unit,
            self.change,
            self.tone,
            self.formatter,
        );
        let node_ref = node_ref();
        let revealed = reveal_once(node_ref);
        let value = Animated::from(
            0.,
            move || if revealed.get() { target } else { 0. },
            Transition::tween(900, Easing::CubicBezier(0.2, 0.7, 0.2, 1.)),
        );
        let value_row = row((
            text(move || formatter(value.get())).style(move || {
                let t = &current_theme();
                style()
                    .font_family(t.get::<Tokens>().display_family.as_str())
                    .font_size(36)
                    .weight(700)
                    .italic(true)
                    .line_height(1.)
            }),
            text(unit).style(style().font_size(16).color(ColorToken::TextMuted)),
        ))
        .style(style().gap(2).align_items(Align::Baseline));
        let trend = self.trend;
        let spark = kivact::view::show(
            move || !trend.get().is_empty(),
            move || {
                node().style(style().size(96, 28)).draw(move |d, s| {
                    let values = trend.get();
                    if values.is_empty() {
                        return;
                    }
                    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
                    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                    let span = (max - min).max(f64::EPSILON);
                    let points: Vec<Point> = values
                        .iter()
                        .enumerate()
                        .map(|(i, y)| {
                            Point::new(
                                i as f32 * s.width / values.len().saturating_sub(1).max(1) as f32,
                                s.height * (1. - ((y - min) / span) as f32),
                            )
                        })
                        .collect();
                    let mut area = Path::new().move_to(Point::new(0., s.height));
                    for point in &points {
                        area = area.line_to(*point);
                    }
                    d.fill_path(
                        &area.line_to(Point::new(s.width, s.height)).close(),
                        d.theme().get::<Tokens>().primary_weak,
                    );
                    d.polyline(points, &Stroke::new(1.5), d.theme().colors().accent);
                })
            },
        );
        self.root
            .child((
                heading,
                value_row,
                row((
                    crate::mono(change).style(move || {
                        let t = &current_theme();
                        style().color(tone.color(t))
                    }),
                    spark,
                ))
                .style(
                    style()
                        .justify_content(Justify::SpaceBetween)
                        .align_items(Align::Center),
                ),
            ))
            .node_ref(node_ref)
            .into_view()
    }
}

/// Skewed segmented meter, with the original 12 × 18 px bars.
#[track_caller]
pub fn gauge(label: impl IntoText, value: impl IntoValue<usize>) -> Gauge {
    let value = value.into_value();
    Gauge {
        root: row(())
            .role(Role::Meter)
            .name(label)
            .value_min(0.)
            .value_max(100.)
            .value_now(move || value.get() as f64),
        value,
        total: 100usize.into_value(),
        tone: Tone::Neutral,
    }
}
/// Presentation builder returned by [`gauge`].
pub struct Gauge {
    root: Node,
    value: Value<usize>,
    total: Value<usize>,
    tone: Tone,
}
impl Gauge {
    /// Reactive number of bars; 100 by default.
    pub fn total(mut self, value: impl IntoValue<usize>) -> Self {
        self.total = value.into_value();
        let total = self.total;
        self.root = self.root.value_max(move || total.get() as f64);
        self
    }
    /// Semantic accent; neutral by default.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}
impl NodeBuilder for Gauge {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Gauge {
    fn into_view(self) -> View {
        let value = self.value;
        let total = self.total;
        let tone = self.tone;

        self.root
            .child(kivact::view::for_index(
                move || (0..total.get()).collect::<Vec<_>>(),
                move |i, _| {
                    node().style(move || {
                        let t = &current_theme();
                        style()
                            .size(12, 18)
                            .transform(Transform::new().skew_x(-12))
                            .background(if i < value.get() {
                                tone.color(t)
                            } else {
                                t.get::<Tokens>().surface_3
                            })
                    })
                },
            ))
            .base_style(style().gap(3).align_items(Align::Center))
            .into_view()
    }
}

/// Segmented circular meter with an upright center label.
#[track_caller]
pub fn ring(label: impl IntoText, value: impl IntoValue<f64>, tone: Tone) -> Node {
    let value = value.into_value();
    let node = node_ref();
    let revealed = reveal_once(node);
    let animated = Animated::from(
        0.,
        move || {
            if revealed.get() {
                value.get().clamp(0., 100.)
            } else {
                0.
            }
        },
        Transition::tween(900, Easing::CubicBezier(0.2, 0.7, 0.2, 1.)),
    );
    column((
        text(move || format!("{:.0}%", value.get())).style(move || {
            let t = &current_theme();
            style()
                .font_family(t.get::<Tokens>().display_family.as_str())
                .font_size(24)
                .weight(700)
                .italic(true)
        }),
        crate::mono(label).style(style().font_size(9).color(ColorToken::TextMuted)),
    ))
    .node_ref(node)
    .role(Role::Meter)
    .value_min(0.)
    .value_max(100.)
    .value_now(value)
    .value_text(move || format!("{:.0}%", value.get()))
    .style(
        style()
            .size(88, 88)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .gap(0),
    )
    .draw(move |d, _| {
        let center = Point::new(44., 44.);
        d.stroke_path(
            &Path::new().arc(center, 36., 0., 359.9, false),
            &Stroke::new(8),
            d.theme().get::<Tokens>().surface_3,
        );
        let end = -90. + animated.get() as f32 * 3.6;
        if end > -90. {
            d.stroke_path(
                &Path::new().arc(center, 36., -90., end, false),
                &Stroke::new(8),
                tone.color(d.theme()),
            );
        }
        d.stroke_path(
            &Path::new().arc(center, 36., 0., 359.9, false),
            &Stroke::new(9).dash([1.2, 8.62]),
            ColorToken::Surface,
        );
    })
}

fn reveal_once(node: NodeRef) -> Signal<bool> {
    let revealed = signal(false);
    effect(move || {
        if !revealed.get_untracked()
            && let Some(rect) = node.rect().get()
        {
            let height = platform::window_size().height;
            let visible = (rect.y + rect.height).min(height) - rect.y.max(0.);
            if visible >= rect.height * 0.3 {
                revealed.set(true);
            }
        }
    });
    revealed
}

/// Cut-corner initials avatar, with optional online status marker.
///
/// `size` is the side in logical pixels, a number rather than a length: the
/// initials and the cut corner are drawn in proportion to it.
#[track_caller]
pub fn avatar(initials: impl IntoText) -> Avatar {
    Avatar {
        root: node(),
        initials: initials.into_text_value(),
        size: 36.,
        color: ColorToken::Accent.into(),
        online: None,
    }
}
/// Presentation builder returned by [`avatar`].
pub struct Avatar {
    root: Node,
    initials: Value<String>,
    size: f32,
    color: Color,
    online: Option<Tone>,
}
impl Avatar {
    /// Side length in logical pixels; 36 by default.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
    /// Avatar fill; the theme accent by default.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    /// Optional status marker; absent by default.
    pub fn online(mut self, tone: Option<Tone>) -> Self {
        self.online = tone;
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
        let initials = self.initials;
        let size = self.size;
        let color = self.color;
        let online = self.online;

        let content = node()
            .style(
                style()
                    .size(size, size)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .background(color)
                    .color(kivact::style::rgb(0xffffff))
                    .clip_path(
                        Shape::polygon()
                            .point(0, 0)
                            .point(pct(100), 0)
                            .point(pct(100), pct(100) - size / 4.)
                            .point(pct(100) - size / 4., pct(100))
                            .point(0, pct(100)),
                    ),
            )
            .child(text(initials).style(move || {
                let t = &current_theme();
                style()
                    .font_family(t.get::<Tokens>().display_family.as_str())
                    .font_size(size * 0.44)
                    .weight(700)
                    .italic(true)
                    .color(kivact::style::rgb(0xffffff))
            }));
        self.root
            .base_style(style().size(size, size))
            .child(content)
            .draw_over(move |d, s| {
                if let Some(tone) = online {
                    d.fill_rect(
                        Rect::new(s.width - 8., s.height - 8., 10., 10.),
                        ColorToken::Surface,
                    );
                    d.fill_rect(
                        Rect::new(s.width - 6., s.height - 6., 6., 6.),
                        tone.color(d.theme()),
                    );
                }
            })
            .into_view()
    }
}

/// Overlapping cut-corner avatars.
#[track_caller]
pub fn avatar_stack(items: impl IntoValue<Vec<(String, Color)>>) -> Node {
    let items = items.into_value();
    row(kivact::view::dynamic(items, |items| {
        items
            .into_iter()
            .enumerate()
            .map(|(i, (name, color))| {
                avatar(name).size(32.).color(color).style(
                    style()
                        .margin_left(if i == 0 { 0 } else { -6 })
                        .border(2, ColorToken::Surface),
                )
            })
            .collect::<Vec<_>>()
    }))
}

/// Overflow count tile matching the avatar stack's cut corner and muted caption.
#[track_caller]
pub fn avatar_more(count: usize) -> Node {
    node()
        .child(
            crate::mono(format!("+{count}"))
                .style(style().font_size(10).color(crate::Color::Text3)),
        )
        .style(
            style()
                .size(32, 32)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .background(crate::Color::Surface3)
                .clip_path(
                    Shape::polygon()
                        .point(0, 0)
                        .point(pct(100), 0)
                        .point(pct(100), pct(100) - 8)
                        .point(pct(100) - 8, pct(100))
                        .point(0, pct(100)),
                ),
        )
}

/// One entry of an [`activity`] timeline.
pub struct ActivityEntry {
    time: String,
    message: View,
    tone: Tone,
}
impl ActivityEntry {
    /// An entry at `time` marked in `tone`. `message` is a plain string or any
    /// application-owned view, such as rich text spans.
    pub fn new(time: impl Into<String>, message: impl IntoView, tone: Tone) -> Self {
        Self {
            time: time.into(),
            message: message.into_view(),
            tone,
        }
    }
}

/// Diamond-marked activity timeline.
/// Entries own non-cloneable views and are consumed once as composition.
#[track_caller]
pub fn activity(entries: Vec<ActivityEntry>) -> Node {
    column(
        entries
            .into_iter()
            .map(
                |ActivityEntry {
                     time,
                     message,
                     tone,
                 }| {
                    column((
                        crate::mono(time).style(style().color(ColorToken::TextMuted)),
                        node().style(style().font_size(12.5)).child(message),
                    ))
                    .style(style().padding_left(26).padding_bottom(14).gap(2))
                    .draw(move |d, s| {
                        d.line(
                            Point::new(5., 7.),
                            Point::new(5., s.height),
                            1.,
                            d.theme().get::<Tokens>().line_strong,
                        );
                        d.fill_path(
                            &Path::new()
                                .move_to(Point::new(5., 2.))
                                .line_to(Point::new(10., 7.))
                                .line_to(Point::new(5., 12.))
                                .line_to(Point::new(0., 7.))
                                .close(),
                            tone.color(d.theme()),
                        );
                    })
                },
            )
            .collect::<Vec<_>>(),
    )
    .role(Role::List)
    .style(style().width(pct(100)))
}

/// Property rows: each key in a 120 px shaded column beside its value, a
/// text or any view, such as a status badge or a copy action (use
/// `.into_view()` to mix them).
/// Values are consumed once as composition; reactive data belongs inside each view.
#[track_caller]
pub fn property_list<V: IntoView>(entries: Vec<(String, V)>) -> Node {
    column(
        entries
            .into_iter()
            .map(|(key, value)| {
                row((
                    node()
                        .style(
                            style()
                                .width(120)
                                .padding_x(10)
                                .padding_y(8)
                                .background(ColorToken::SurfaceAlt),
                        )
                        .child(crate::mono(key)),
                    node()
                        .child(value)
                        .style(style().grow(1).padding_x(10).padding_y(8).font_size(12.5)),
                ))
                .style(style().border_bottom(1, ColorToken::Border))
            })
            .collect::<Vec<_>>(),
    )
    .style(style().width(pct(100)).border_top(1, ColorToken::Border))
}
