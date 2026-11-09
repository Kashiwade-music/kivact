//! Role: Count a console KPI tile on the standard statistic up once it scrolls into view.
use crate::StatVariant;
use kivact::interaction::Tone;
use kivact::motion::{Animated, Easing, Transition};
use kivact::state::{IntoText, IntoValue, Machine, Step, Value, effect, machine, memo};
use kivact::style::{Align, ColorToken, style};
use kivact::view::{IntoView, Node, NodeBuilder, NodeRef, Role, View, node, node_ref, row, text};

/// A KPI tile: a standard `kivact::ui::stat` in [`StatVariant::Kpi`] whose
/// number counts up from zero once the tile is in view. Created by [`kpi`].
pub struct Kpi {
    root: Node,
    label: Value<String>,
    target: f64,
    unit: Value<String>,
    // xtask-allow(state-fields): builder option - change
    change: Option<Value<String>>,
    tone: Tone,
    // xtask-allow(state-fields): builder option - heading
    heading: Option<View>,
    // xtask-allow(state-fields): builder option - trend
    trend: Option<Value<Vec<f64>>>,
    format: Box<dyn Fn(f64) -> String>,
}

/// A KPI tile titled `label` that counts up to `target` over 900 ms when at
/// least 30% of it is in the window (at once under reduced motion).
///
/// Assistive technology reads the settled value with its unit, not the
/// counting one. The default format has no decimals and no grouping.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_game as game;
///
/// fn uptime() -> impl IntoView {
///     game::kpi("Uptime", 99.97)
///         .decimals(2)
///         .unit("%")
///         .change("+0.02pt")
///         .tone(Tone::Success)
///         .trend(vec![0.2, 0.4, 0.3, 0.7])
/// }
/// ```
#[track_caller]
pub fn kpi(label: impl IntoText, target: f64) -> Kpi {
    Kpi {
        root: node(),
        label: label.into_text_value(),
        target,
        unit: String::new().into_value(),
        change: None,
        tone: Tone::Neutral,
        heading: None,
        trend: None,
        format: Box::new(|value| format!("{value:.0}")),
    }
}

impl Kpi {
    /// Decimal places of the default number format; zero by default.
    pub fn decimals(mut self, decimals: usize) -> Self {
        self.format = Box::new(move |value| format!("{value:.decimals$}"));
        self
    }
    /// Unit suffix after the number, in smaller muted text; empty by default.
    pub fn unit(mut self, unit: impl IntoText) -> Self {
        self.unit = unit.into_text_value();
        self
    }
    /// Change caption under the number, in the tone's color.
    pub fn change(mut self, change: impl IntoText) -> Self {
        self.change = Some(change.into_text_value());
        self
    }
    /// Semantic accent of the top rule and the change caption; neutral by default.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
    /// An application-owned heading in place of the label text; the label
    /// stays the accessible name.
    pub fn heading(mut self, heading: impl IntoView) -> Self {
        self.heading = Some(heading.into_view());
        self
    }
    /// Trend samples drawn as a [`sparkline`](crate::sparkline) beside the change caption.
    pub fn trend(mut self, trend: impl IntoValue<Vec<f64>>) -> Self {
        self.trend = Some(trend.into_value());
        self
    }
    /// Application-defined number text, replacing [`Self::decimals`].
    pub fn format(mut self, format: impl Fn(f64) -> String + 'static) -> Self {
        self.format = Box::new(format);
        self
    }
}

impl NodeBuilder for Kpi {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Kpi {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            target,
            unit,
            change,
            tone,
            heading,
            trend,
            format,
        } = self;
        let tile = node_ref();
        let revealed = reveal_once(tile);
        let value = Animated::from(
            0.0,
            move || if revealed.get() { target } else { 0.0 },
            Transition::tween(900, Easing::CubicBezier(0.2, 0.7, 0.2, 1.)),
        );
        let settled = format(target);
        let number = memo(move || format(value.get()));
        let mut stat = kivact::ui::stat(label, move || format!("{settled}{}", unit.get()))
            .variant(StatVariant::Kpi)
            .variant(tone)
            .value_content(
                row((
                    text(number),
                    text(unit).style(style().font_size(16).color(ColorToken::TextMuted)),
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
            stat = stat.chart(crate::sparkline(trend));
        }
        // The caller's modifiers apply to the statistic's own group root.
        *stat.node_mut() = root.role(Role::Group);
        stat.node_ref(tile).into_view()
    }
}

/// Whether the tile has been revealed; it never hides again.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reveal {
    Hidden,
    Shown,
}

struct RevealMachine;
impl Machine for RevealMachine {
    type State = Reveal;
    type Event = bool;
    type Effect = ();
    fn transition(&self, state: &Reveal, visible: bool) -> Step<Reveal, bool, ()> {
        match state {
            Reveal::Hidden if visible => Step::to(Reveal::Shown),
            _ => Step::stay(),
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
