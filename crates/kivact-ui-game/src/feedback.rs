//! Role: Banners, alerts, toast cards, skeletons and empty states.
use crate::Color;
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::draw::{Layer, Path, Point, Rect};
use kivact::motion::{FrameRate, frame_loop};
use kivact::platform;
use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use kivact::style::{Align, BorderStyle, ColorToken, Justify, pct, style};
use kivact::view::HandlerResult;
use kivact::view::{IntoText, IntoValue, IntoView, Role, View, callback, node_ref};
use kivact::{Value, signal};

use crate::Tokens;
use crate::display::ToneExt;
use kivact::ui::Tone;

/// Full-width system banner. Created by [`banner`].
pub struct Banner {
    root: Node,
    title: Value<String>,
    content: Option<crate::slot::Slot>,
    tone: Tone,
    action: View,
}

/// Full-width system banner: a striped `title` tape in `tone`, the message
/// ([`Banner::content`]) and the application's `action` area at the end.
#[track_caller]
pub fn banner(title: impl IntoText, tone: Tone, action: impl IntoView) -> Banner {
    Banner {
        root: row(()).style(
            style()
                .height(44)
                .padding_right(14)
                .gap(14)
                .align_items(Align::Center)
                .background(Color::Navy)
                .color(kivact::style::rgb(0xffffff)),
        ),
        title: title.into_text_value(),
        content: None,
        tone,
        action: action.into_view(),
    }
}
impl Banner {
    /// The message, built once by `content` when the banner mounts: a text
    /// or any view, such as rich text spans. It keeps the banner's text style
    /// and fills the available width; without it the banner shows only its
    /// tape and its action.
    pub fn content<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.content = Some(crate::slot::slot(content));
        self
    }
}
impl NodeBuilder for Banner {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Banner {
    fn into_view(self) -> View {
        let Self {
            root,
            title,
            content,
            tone,
            action,
        } = self;
        let tape = node()
            .style(
                style()
                    .padding_x(12)
                    .height(pct(100))
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .color(Color::Navy),
            )
            .draw(move |d, s| {
                d.layer(Layer::new().clip_rect(s.to_rect()), |d| {
                    let color = if tone == Tone::Warning {
                        d.theme().get::<Tokens>().yellow
                    } else {
                        tone.color(d.theme())
                    };
                    d.fill_rect(s.to_rect(), color);
                    let mut x = -44.;
                    while x < s.width + 44. {
                        d.fill_path(
                            &Path::new()
                                .move_to(Point::new(x, s.height))
                                .line_to(Point::new(x + s.height, 0.))
                                .line_to(Point::new(x + s.height + 8., 0.))
                                .line_to(Point::new(x + 8., s.height))
                                .close(),
                            kivact::style::rgb(0x1a1a1a),
                        );
                        x += 16.;
                    }
                });
            })
            .child(crate::mono(title).style(move || {
                let t = &current_theme();
                style()
                    .font_size(10)
                    .weight(800)
                    .padding_x(6)
                    .padding_y(4)
                    .background(if tone == Tone::Warning {
                        t.get::<Tokens>().yellow
                    } else {
                        tone.color(t)
                    })
            }));
        root.child((
            tape,
            node()
                .style(
                    style()
                        .font_size(12.5)
                        .grow(1)
                        .color(kivact::style::rgb(0xffffff)),
                )
                .child(content.map(|content| content())),
            action,
        ))
        .role(Role::Status)
        .into_view()
    }
}

/// Inline alert whose dismissal is owned by the application.
pub struct Alert {
    root: Node,
    title: Value<String>,
    message: Value<String>,
    actions: Option<crate::slot::Slot>,
    tone: Tone,
    on_dismiss: Option<kivact::Callback<()>>,
}
/// Create an inline alert. [`Alert::actions`] adds application-owned action
/// controls under the message.
#[track_caller]
pub fn alert(title: impl IntoText) -> Alert {
    Alert {
        root: row(()),
        title: title.into_text_value(),
        message: String::new().into_text_value(),
        actions: None,
        tone: Tone::Neutral,
        on_dismiss: None,
    }
}

impl Alert {
    /// Supporting text; empty by default.
    pub fn description(mut self, value: impl IntoText) -> Self {
        self.message = value.into_text_value();
        self
    }
    /// Semantic accent; neutral by default.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Application-owned action controls under the message, built once by
    /// `actions` when the alert mounts.
    pub fn actions<V: IntoView>(mut self, actions: impl FnOnce() -> V + 'static) -> Self {
        self.actions = Some(crate::slot::slot(actions));
        self
    }
    /// Report dismissal; the application decides whether to hide the alert.
    pub fn on_dismiss<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.on_dismiss = Some(callback(move |_| f()));
        self
    }
}
impl NodeBuilder for Alert {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Alert {
    fn into_view(mut self) -> View {
        if self.root.accessible_role().is_none() {
            self.root = self
                .root
                .role(if matches!(self.tone, Tone::Warning | Tone::Danger) {
                    Role::Alert
                } else {
                    Role::Status
                });
        }
        let tone = self.tone;
        let body = row((
            kivact::control::icon(match tone {
                Tone::Success => "circle-check",
                Tone::Warning => "alert-triangle",
                Tone::Danger => "alert-octagon",
                _ => "info-circle",
            })
            .style(move || {
                let t = &current_theme();
                style().size(18, 18).color(tone.color(t))
            }),
            column((
                text(self.title).style(style().font_size(13).weight(700)),
                text(self.message).style(style().font_size(12.5).color(ColorToken::TextMuted)),
                self.actions.map(|actions| actions()),
            ))
            .style(style().grow(1).gap(2)),
        ))
        .style(style().gap(12).grow(1).align_items(Align::Start));
        let close = self.on_dismiss.map(|callback| {
            crate::tool_button("×")
                .name(kivact::i18n::message("game.alert.close", "Close alert"))
                .on_press(move || callback.call(()))
        });
        self.root
            .base_style(move || {
                let t = &current_theme();
                style()
                    .padding_y(12)
                    .padding_x(14)
                    .gap(12)
                    .align_items(Align::Start)
                    .background(tone.weak(t))
                    .border(1, tone.color(t).mix(t.colors().surface, 0.65))
                    .border_left(4, tone.color(t))
            })
            .child((body, close))
            .live_region(if matches!(tone, Tone::Warning | Tone::Danger) {
                kivact::view::LiveRegion::Assertive
            } else {
                kivact::view::LiveRegion::Polite
            })
            .into_view()
    }
}

/// Skeleton rows with a decorative shimmer that honors reduced motion.
#[track_caller]
pub fn skeleton() -> Skeleton {
    Skeleton {
        root: column(())
            .role(Role::Status)
            .name(kivact::i18n::message("game.skeleton.label", "Loading")),
        rows: 1usize.into_value(),
    }
}
/// Presentation builder returned by [`skeleton`].
pub struct Skeleton {
    root: Node,
    rows: Value<usize>,
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

        let phase = signal(0_f64);
        let node_ref = node_ref();
        let clock = frame_loop(move |f| phase.set((f.now.as_secs_f64() / 1.4).fract()));
        kivact::effect(move || {
            let visible = node_ref.rect().get().is_some_and(|rect| {
                rect.y < platform::window_size().height && rect.y + rect.height > 0.
            });
            if visible && !platform::reduced_motion() && platform::surface_active() {
                clock.start(FrameRate::Hz(30.));
            } else {
                clock.stop();
            }
        });
        let block = move |w, h| {
            node().style(style().size(w, h)).draw(move |d, s| {
                d.layer(Layer::new().clip_rect(s.to_rect()), |d| {
                    let p = d.theme().get::<Tokens>();
                    d.fill_rect(s.to_rect(), p.surface_3);
                    let x = s.width * (phase.get() * 3. - 1.);
                    d.fill_rect(
                        Rect::new(x, 0., s.width * 0.3, s.height),
                        kivact::draw::linear_gradient(100).colors([
                            p.surface_3,
                            d.theme().colors().surface_alt,
                            p.surface_3,
                        ]),
                    );
                });
            })
        };
        self.root
            .child(kivact::view::for_index(
                move || (0..rows.get()).collect::<Vec<_>>(),
                move |_, _| {
                    row((
                        block(36, 36).style(style().clip_path(crate::cut_corner(8.))),
                        column((block(160, 10), block(230, 10))).style(style().gap(8)),
                    ))
                    .style(style().gap(12).align_items(Align::Center))
                },
            ))
            .node_ref(node_ref)
            .base_style(style().gap(14).width(pct(100)))
            .into_view()
    }
}

/// Dashed empty-state panel with an application-owned action.
#[track_caller]
pub fn empty_state(title: impl IntoText) -> EmptyState {
    EmptyState {
        root: column(()).role(Role::Group),
        title: title.into_text_value(),
        description: String::new().into_text_value(),
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
                node().base_style(style().size(64, 64)).draw(|d, _| {
                    let color = d.theme().get::<Tokens>().text_3;
                    d.stroke_path(
                        &kivact::draw::Path::new().arc(Point::new(32., 32.), 28., 0., 359.9, false),
                        &kivact::draw::Stroke::new(2.).dash([10., 4., 3., 4.]),
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
                        &kivact::draw::Stroke::new(2.),
                        color,
                    );
                    d.line(
                        Point::new(28., 33.),
                        Point::new(36., 33.),
                        2.5,
                        ColorToken::Accent,
                    );
                }),
                text(title).base_style(move || {
                    let t = &current_theme();
                    style()
                        .font_family(t.get::<Tokens>().display_family.as_str())
                        .font_size(20)
                        .weight(700)
                        .italic(true)
                }),
                text(description).base_style(
                    style()
                        .font_size(12.5)
                        .max_width(300)
                        .color(ColorToken::TextMuted),
                ),
                action.map(|slot| slot()),
            ))
            .base_style(
                style()
                    .width(pct(100))
                    .padding_y(28)
                    .padding_x(16)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .gap(8)
                    .border(1.5, ColorToken::Border)
                    .border_style(BorderStyle::Dashed),
            )
            .into_view()
    }
}
