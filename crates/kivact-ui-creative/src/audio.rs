//! Role: Transport, meter and mixer strip controls of the Creative kit.
pub(crate) mod view_state;
use kivact::ui::{ButtonVariant, ListItem};
use kivact::view::HandlerResult;

use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::draw::Rect;
use kivact::i18n::message;
use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use kivact::style::{Align, Color, ColorToken, Direction, Justify, pct, style};
use kivact::view::{IntoText, IntoValue, IntoView, Orientation, Role, View, callback};
use kivact::{Binding, Callback, Value, effect, memo, signal};

use crate::{fader, knob};

/// Playback button state; the application owns its clock.
#[derive(Clone, Debug, PartialEq)]
pub struct TransportState {
    pub playing: bool,
    pub recording: bool,
    pub cycling: bool,
}
impl Default for TransportState {
    fn default() -> Self {
        Self {
            playing: false,
            recording: false,
            cycling: true,
        }
    }
}

/// Playback controls without a clock. The application handles seeking.
pub struct Transport {
    root: Node,
    binding: Binding<TransportState>,
    to_start: Option<Callback<()>>,
}
/// Create controls for the application's playback state.
#[track_caller]
pub fn transport(binding: impl Into<Binding<TransportState>>) -> Transport {
    Transport {
        root: row(())
            .role(Role::Toolbar)
            .name(message("creative.transport.label", "Transport"))
            .style(
                style()
                    .align_items(Align::Center)
                    .gap(2)
                    .padding(3)
                    .radius(9)
                    .background(crate::Color::Canvas),
            ),
        binding: binding.into(),
        to_start: None,
    }
}
impl Transport {
    /// Report a request to return to the application's start position.
    pub fn on_to_start<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.to_start = Some(callback(move |_| f()));
        self
    }
}
impl NodeBuilder for Transport {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Transport {
    fn into_view(self) -> View {
        transport_view(self.root, self.binding, self.to_start).into_view()
    }
}
fn transport_view(
    root: Node,
    binding: Binding<TransportState>,
    to_start: Option<Callback<()>>,
) -> Node {
    let state = binding.into_value();
    let modify = move |f: fn(&mut TransportState)| {
        let mut next = state.get_untracked();
        f(&mut next);
        binding.set(next);
    };
    root.child((
        to_start.map(|callback| {
            kivact::ui::button(message("creative.transport.start", "To Start"))
                .hide_label(true)
                .icon(kivact::control::icon("start"))
                .variant(ButtonVariant::Ghost)
                .on_press(move || callback.call(()))
        }),
        kivact::ui::button(message("creative.transport.stop", "Stop"))
            .hide_label(true)
            .icon(kivact::control::icon("stop"))
            .variant(ButtonVariant::Ghost)
            .on_press(move || modify(|s| s.playing = false)),
        kivact::ui::button(message("creative.transport.play", "Play"))
            .hide_label(true)
            .icon(kivact::control::icon("play"))
            .variant(ButtonVariant::Ghost)
            .selected(move || state.get().playing)
            .on_press(move || modify(|s| s.playing = !s.playing)),
        kivact::ui::button(message("creative.transport.record", "Record"))
            .hide_label(true)
            .icon(kivact::control::icon("record"))
            .variant(ButtonVariant::Ghost)
            .selected(move || state.get().recording)
            .style(style().color(crate::Color::X))
            .on_press(move || modify(|s| s.recording = !s.recording)),
        kivact::ui::button(message("creative.transport.cycle", "Cycle"))
            .hide_label(true)
            .icon(kivact::control::icon("repeat"))
            .variant(ButtonVariant::Ghost)
            .selected(move || state.get().cycling)
            .on_press(move || modify(|s| s.cycling = !s.cycling)),
    ))
}

/// An audio meter with clip hold and accessible numeric level.
pub struct Meter {
    root: Node,
    level: Value<f64>,
    horizontal: Value<bool>,
    channels: usize,
    segmented: Value<bool>,
}
/// An audio meter receiving normalized linear amplitude (0..1, >1 clips).
#[track_caller]
pub fn meter(label: impl IntoText, level: impl IntoValue<f64>) -> Meter {
    Meter {
        root: node()
            .role(Role::Meter)
            .name(label)
            .value_min(0.0)
            .value_max(1.0)
            .style(
                style()
                    .size(30, 180)
                    .background(crate::Color::Canvas)
                    .radius(5),
            ),
        level: level.into_value(),
        horizontal: false.into_value(),
        channels: 2,
        segmented: false.into_value(),
    }
}
impl Meter {
    /// Choose separated LED cells rather than the source's continuous bar
    /// while true (constant or reactive).
    #[track_caller]
    pub fn segmented(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.segmented = enabled.into_value();
        self
    }
    /// Lay the channels out as a horizontal strip
    /// ([`Orientation::Horizontal`]) rather than the default vertical channel
    /// meter (constant or reactive).
    #[track_caller]
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        let orientation = orientation.into_value();
        let enabled = memo(move || orientation.get() == Orientation::Horizontal);
        self.horizontal = enabled;
        self.root = self.root.style(move || {
            if enabled.get() {
                style().size(240, 18)
            } else {
                style().size(30, 180)
            }
        });
        self
    }
    /// Display one to eight channels.
    pub fn channels(mut self, channels: usize) -> Self {
        self.channels = channels.clamp(1, 8);
        self
    }
}
impl NodeBuilder for Meter {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Meter {
    fn into_view(self) -> View {
        let Self {
            root,
            level,
            horizontal,
            channels,
            segmented,
        } = self;
        let clipped = signal(false);
        effect(move || {
            if level.get() > 1.0 {
                clipped.set(true);
            }
        });
        root.value_now(move || level.get().clamp(0.0, 1.0))
            .value_text(move || format!("{:.1} dB", 20.0 * level.get().max(0.001).log10()))
            .orientation(move || {
                if horizontal.get() {
                    Orientation::Horizontal
                } else {
                    Orientation::Vertical
                }
            })
            .on_press(move || clipped.set(false))
            .draw(move |d, size| {
                let (horizontal, segmented) = (horizontal.get(), segmented.get());
                let p = d.theme().get::<crate::tokens::Tokens>();
                let v = level.get().clamp(0.0, 1.0) as f32;
                let extent = if horizontal {
                    size.width - 10.0
                } else {
                    size.height - 12.0
                };
                let width = if horizontal {
                    (size.height - 4.0) / channels as f32
                } else {
                    (size.width - 4.0) / channels as f32
                };
                for ch in 0..channels {
                    let amp = v;
                    for i in 0..40 {
                        let n = i as f32 / 40.0;
                        let color = if n >= amp {
                            Color::token(ColorToken::Raised)
                        } else if n > 0.9 {
                            p.x
                        } else if n > 0.75 {
                            p.warning
                        } else {
                            p.success
                        };
                        let r = if horizontal {
                            Rect::new(
                                2.0 + extent * n,
                                2.0 + ch as f32 * width,
                                (extent / 40.0 - if segmented { 1.0 } else { 0.0 }).max(1.0),
                                width - 1.0,
                            )
                        } else {
                            Rect::new(
                                2.0 + ch as f32 * width,
                                size.height - 2.0 - extent * (n + 0.025),
                                width - 1.0,
                                (extent / 40.0 - if segmented { 1.0 } else { 0.0 }).max(1.0),
                            )
                        };
                        d.fill_rect(r, color);
                    }
                }
                let clip = if horizontal {
                    Rect::new(size.width - 6.0, 2.0, 4.0, size.height - 4.0)
                } else {
                    Rect::new(2.0, 2.0, size.width - 4.0, 4.0)
                };
                d.rounded_rect(
                    clip,
                    2.0,
                    if clipped.get() {
                        p.x
                    } else {
                        p.x.with_alpha(0.17)
                    },
                );
            })
            .into_view()
    }
}

/// A channel strip with application-provided routing, inserts and meter level.
pub struct MixerStrip {
    root: Node,
    label: String,
    color: Color,
    channel: crate::ChannelState,
    gain_range: (f64, f64),
    level: Value<f64>,
    inserts: Value<Vec<String>>,
    routing: Option<(Value<Vec<String>>, Binding<usize>)>,
}
/// A mixer channel editing the gain, pan, mute and solo of `channel`; the
/// application supplies levels, the gain range and routing.
#[track_caller]
pub fn mixer_strip(
    label: impl Into<String>,
    color: Color,
    channel: crate::ChannelState,
) -> MixerStrip {
    let label = label.into();
    MixerStrip {
        root: node().role(Role::Group).name(label.clone()).style(move || {
            let t = &current_theme();
            let p = t.get::<crate::tokens::Tokens>();
            style()
                .direction(Direction::Column)
                .width(94)
                .shrink(0)
                .background(p.island)
                .border(1, p.hair)
                .radius(14)
                .overflow(kivact::style::Overflow::Hidden)
                .selected(style().background(p.accent.with_alpha(0.07)))
                .color(p.text)
        }),
        label,
        color,
        channel,
        gain_range: (0.0, 1.0),
        level: 0.0.into_value(),
        inserts: Vec::new().into_value(),
        routing: None,
    }
}
impl MixerStrip {
    /// Configure the gain fader's inclusive bounds; the default is 0..1.
    /// Non-finite, empty or reversed ranges are ignored.
    pub fn gain_range(mut self, min: f64, max: f64) -> Self {
        if min.is_finite() && max.is_finite() && max > min {
            self.gain_range = (min, max);
        }
        self
    }
    /// Final meter amplitude, displayed directly without applying gain or mute.
    pub fn level(mut self, level: impl IntoValue<f64>) -> Self {
        self.level = level.into_value();
        self
    }
    /// Display insert names without disconnected bypass controls.
    pub fn inserts(mut self, names: impl IntoValue<Vec<String>>) -> Self {
        self.inserts = names.into_value();
        self
    }
    /// Show an input routing selector choosing among `routes` by index into
    /// `route`. Without routes, or without this call, the strip has no
    /// selector.
    pub fn routes(
        mut self,
        routes: impl IntoValue<Vec<String>>,
        route: impl Into<Binding<usize>>,
    ) -> Self {
        self.routing = Some((routes.into_value(), route.into()));
        self
    }
}
impl NodeBuilder for MixerStrip {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for MixerStrip {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            color,
            channel:
                crate::ChannelState {
                    gain,
                    pan,
                    mute,
                    solo,
                },
            gain_range,
            level,
            inserts,
            routing,
        } = self;
        let insert_views = kivact::view::for_index(inserts, |_, name| {
            text(name).style(
                style()
                    .width(pct(100))
                    .height(21)
                    .padding_x(5)
                    .font_size(10),
            )
        });
        let routing = routing.map(|(routes, route)| {
            let route_binding = route.optional();
            kivact::ui::select(
                message("creative.mixer.routing", "Input routing"),
                move || {
                    routes
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, name)| ListItem::new(i, name))
                        .collect::<Vec<_>>()
                },
                route_binding,
            )
            .placeholder("")
            .hidden(move || routes.with(Vec::is_empty))
            .style(
                style()
                    .width(pct(100))
                    .height(21)
                    .font_size(10)
                    .padding_x(4),
            )
        });
        let mute_label = message("creative.mixer.mute", "M");
        let solo_label = message("creative.mixer.solo", "S");
        let tiny = |label: Value<String>, state: Binding<bool>| {
            kivact::ui::toggle(label, state)
                .style(style().size(18, 18).padding(0).font_size(9).radius(4))
        };
        let content = column((
            node().style(style().height(3).background(color)),
            column((routing, column(insert_views).style(style().gap(2))))
                .style(style().padding(6).gap(3)),
            knob(message("creative.mixer.pan", "Pan"), pan)
                .range(-100.0, 100.0)
                .precision(0)
                .step(1.0)
                .bipolar(true)
                .accent_color(color)
                .style(style().size(40, 76).align_self(Align::Center)),
            row((tiny(mute_label, mute), tiny(solo_label, solo)))
                .style(style().gap(2).justify_content(Justify::Center)),
            row((
                fader(message("creative.mixer.volume", "Volume"), gain)
                    .range(gain_range.0, gain_range.1)
                    .style(style().size(36, 184)),
                meter(message("creative.mixer.peak", "Peak"), level).style(style().size(14, 184)),
            ))
            .style(style().padding(6).gap(6).justify_content(Justify::Center)),
            text(label).style(
                style()
                    .height(24)
                    .text_align(kivact::style::TextAlign::Center)
                    .weight(600)
                    .background(color.with_alpha(0.16))
                    .radius(6)
                    .margin(6),
            ),
        ));
        root.child(content).into_view()
    }
}
