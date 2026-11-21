//! Role: The mixer_strip component: one channel's controls, level and routing.

use super::meter::meter;
use crate::color::{ACCENT, HAIR, ISLAND, TEXT};
use crate::{ChannelState, fader, knob};
use kivact::component;
use kivact::i18n::message;
use kivact::interaction::{ListItem, ScalarRange};
use kivact::state::{Signal, Value};
use kivact::style::{Align, Color, FlexDirection, Justify, Overflow, pct, style};
use kivact::view::{Element, Role, column, node, row, text};

/// A channel strip with application-provided routing, inserts and meter level.
///
/// `channel` is the strip's gain, pan, mute and solo. `level` is the final
/// meter amplitude, shown without applying gain or mute. `gain_range` bounds
/// the gain fader (default 0..1). `inserts` lists insert names without
/// disconnected bypass controls. `routes` lists the input routes by name and
/// `route` is the index into it that the routing selector shows and writes;
/// without `route` the strip has no selector, and empty `routes` hides it.
#[component]
pub fn mixer_strip(
    label: Value<String>,
    color: Value<Color>,
    channel: ChannelState,
    #[prop(default = ScalarRange::new(0.0, 1.0).expect("invariant: default gain range"))]
    gain_range: Value<ScalarRange>,
    #[prop(default = 0.0)] level: Value<f64>,
    #[prop(default = Vec::<String>::new())] inserts: Value<Vec<String>>,
    #[prop(default = Vec::<String>::new())] routes: Value<Vec<String>>,
    route: Option<Signal<usize>>,
) -> impl Element {
    let ChannelState {
        gain,
        pan,
        mute,
        solo,
    } = channel;
    let root = node().role(Role::Group).name(label).style(
        style()
            .flex_direction(FlexDirection::Column)
            .width(94)
            .flex_shrink(0.0)
            .background(ISLAND)
            .border(1, HAIR)
            .border_radius(14)
            .overflow(Overflow::Hidden)
            .selected(style().background(ACCENT.with_alpha(0.07)))
            .color(TEXT),
    );
    let insert_views = kivact::view::for_index(inserts, |_, name| {
        text(name).style(
            style()
                .width(pct(100))
                .height(21)
                .padding_x(5)
                .font_size(10),
        )
    });
    let routing = routing_control(route.map(|route| (routes, route)));
    let mute_label = message("creative.mixer.mute", "M");
    let solo_label = message("creative.mixer.solo", "S");
    let tiny = |label: Value<String>, state: Signal<bool>| {
        kivact::ui::toggle_button(label, state).with_node(move |node| {
            node.style(
                style()
                    .size(18, 18)
                    .padding(0)
                    .font_size(9)
                    .border_radius(4),
            )
        })
    };
    let content = column((
        node().style(move || style().height(3).background(color.get())),
        column((routing, column(insert_views).style(style().gap(2))))
            .style(style().padding(6).gap(3)),
        knob(message("creative.mixer.pan", "Pan"), pan)
            .range(
                ScalarRange::new(-100.0, 100.0).expect("invariant: finite ordered control range"),
            )
            .precision(0)
            .step(1.0)
            .bipolar(true)
            .accent_color(color)
            .with_node(move |node| node.style(style().size(40, 76).align_self(Align::Center))),
        row((tiny(mute_label, mute), tiny(solo_label, solo)))
            .style(style().gap(2).justify_content(Justify::Center)),
        row((
            fader(message("creative.mixer.volume", "Volume"), gain)
                .range(gain_range)
                .with_node(move |node| node.style(style().size(36, 184))),
            meter(message("creative.mixer.peak", "Peak"), level)
                .with_node(move |node| node.style(style().size(14, 184))),
        ))
        .style(style().padding(6).gap(6).justify_content(Justify::Center)),
        text(label).style(move || {
            style()
                .height(24)
                .text_align(kivact::style::TextAlign::Center)
                .font_weight(600)
                .background(color.get().with_alpha(0.16))
                .border_radius(6)
                .margin(6)
        }),
    ));
    root.child(content)
}

/// Render the caller-owned optional input-route selector.
fn routing_control(
    routing: Option<(Value<Vec<String>>, Signal<usize>)>,
) -> Option<kivact::ui::Select<usize>> {
    routing.map(|(routes, route)| {
        let route_binding = route.optional();
        kivact::ui::select(
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
        .label(message("creative.mixer.routing", "Input routing"))
        .placeholder("")
        .with_node(move |node| node.hidden(move || routes.with(Vec::is_empty)))
        .with_node(move |node| {
            node.style(
                style()
                    .width(pct(100))
                    .height(21)
                    .font_size(10)
                    .padding_x(4),
            )
        })
    })
}
