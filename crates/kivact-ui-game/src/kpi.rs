//! Role: Count a console KPI tile on the standard statistic up once it scrolls into view.
use crate::variants::StatVariant;
use kivact::component;
use kivact::interaction::Tone;
use kivact::motion::{Animated, Easing, Transition};
use kivact::state::{Value, effect, machine, memo};
use kivact::style::color::TEXT_MUTED;
use kivact::style::{Align, style};
use kivact::view::{Element, NodeRef, View, node_ref, row, text};
use std::rc::Rc;

/// A KPI tile titled `label` that counts up to `target` over 900 ms when at
/// least 30% of it is in the window (at once under reduced motion). The
/// `unit` follows the number in smaller muted text, `change` is the caption
/// under it in the tone's color, and `trend` draws a sparkline beside it.
///
/// Assistive technology reads the settled value with its unit, not the
/// counting one. The default format has no decimals and no grouping.
#[component]
pub fn kpi(
    label: Value<String>,
    target: f64,
    #[prop(default = 0usize)] decimals: usize,
    #[prop(default = String::new())] unit: Value<String>,
    change: Option<Value<String>>,
    #[prop(default = Tone::Neutral)] tone: Value<Tone>,
    heading: Option<View>,
    trend: Option<Value<Vec<f64>>>,
    format: Option<Rc<dyn Fn(f64) -> String>>,
) -> impl Element {
    let format: Rc<dyn Fn(f64) -> String> = match format {
        Some(format) => format,
        None => Rc::new(move |value| format!("{value:.decimals$}")),
    };
    let tile = node_ref();
    let revealed = reveal_once(tile);
    let value = Animated::from(
        0.0,
        move || if revealed.get() { target } else { 0.0 },
        Transition::tween(900, Easing::CubicBezier(0.2, 0.7, 0.2, 1.)),
    );
    let settled = format(target);
    let number = memo({
        let format = Rc::clone(&format);
        move || format(value.get())
    });
    let text_value = memo(move || format!("{settled}{}", unit.get()));
    let mut stat = kivact::ui::stat(label, text_value)
        .variant(StatVariant::Kpi)
        .variant(tone.get())
        .value_content(
            row((
                text(number),
                text(unit).style(style().font_size(16).color(TEXT_MUTED)),
            ))
            .style(style().gap(2).align_items(Align::Baseline)),
        );
    if let Some(heading) = heading {
        stat = stat.label_content(heading);
    }
    if let Some(change) = change {
        stat = stat.delta(change);
    }
    if let Some(trend) = trend {
        stat = stat.chart(kivact::ui::sparkline(trend));
    }
    stat.node_ref(tile)
}

/// Whether the tile has been revealed; it never hides again.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reveal {
    Hidden,
    Shown,
}

struct RevealMachine;

impl kivact::state::Machine for RevealMachine {
    type State = Reveal;
    type Event = bool;
    type Effect = ();
    fn transition(&self, state: &Reveal, visible: bool) -> kivact::state::Step<Reveal, bool, ()> {
        match state {
            Reveal::Hidden if visible => kivact::state::Step::to(Reveal::Shown),
            _ => kivact::state::Step::stay(),
        }
    }
}

/// Become true once at least 30% of `tile`'s height is inside the window.
fn reveal_once(tile: NodeRef) -> Value<bool> {
    let reveal = machine(Reveal::Hidden, || RevealMachine, |_, ()| {});
    let rect = tile.rect();
    let window = kivact::platform::window_size();
    effect(move || {
        if let Some(rect) = rect.get() {
            let visible = (rect.y + rect.height).min(window.get().height) - rect.y.max(0.0);
            reveal.send(visible >= rect.height * 0.3);
        }
    });
    let state = reveal.state();
    memo(move || state.get() == Reveal::Shown)
}
