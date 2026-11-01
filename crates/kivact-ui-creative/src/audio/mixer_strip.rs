//! Role: The MixerStrip builder: one channel's controls, level and routing.

use super::meter::meter;
use crate::{fader, knob};
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::i18n::message;
use kivact::style::current_theme;
use kivact::style::{Align, Color, Direction, Justify, pct, style};
use kivact::ui::ListItem;
use kivact::view::{IntoValue, IntoView, Role, View};
use kivact::{Binding, Value};

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
