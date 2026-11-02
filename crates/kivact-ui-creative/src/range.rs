//! Role: Two-endpoint range control of the Creative kit.

use kivact::control::{Node, NodeBuilder, node};
use kivact::draw::{Rect, RegionId};
use kivact::i18n::{interpolate, message};
use kivact::interaction::{RangeThumb, use_range_scalar};
use kivact::primitive::{NodeExt, PartState};
use kivact::style::{Cursor, pct, style};
use kivact::ui::{EditPhase, ScalarRange};
use kivact::view::{
    A11yAction, A11yEvent, A11yRequest, Handled, HandlerResult, IntoText, IntoValue, IntoView, Key,
    Orientation, Role, View, callback,
};
use kivact::{Binding, Callback, Value, binding, effect, memo, signal, var};

/// The side of an end's square region, which the pointer grabs.
const THUMB: f32 = 20.0;

/// What the keys and actions on the root of a range edit.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RangePart {
    Start,
    End,
    Body,
}

impl RangePart {
    /// The region its handle draws.
    fn region(self) -> RegionId {
        RegionId(match self {
            RangePart::Start => 1,
            RangePart::End => 2,
            RangePart::Body => 3,
        })
    }
}

/// Two ordered endpoints, edited without crossing or hiding the other endpoint.
pub struct Range {
    root: Node,
    label: Value<String>,
    state: Binding<(f64, f64)>,
    min: f64,
    max: f64,
    step: f64,
    interval: Option<Value<kivact::style::Color>>,
    movable: Value<bool>,
    on_edit: Option<Callback<EditPhase>>,
}
/// Two endpoints of `value` on one track, start before end: the
/// [`use_range_scalar`] of the [`limits`](Range::limits) and the
/// [`step`](Range::step), drawn on a canvas.
///
/// The control keeps one keyboard focus: the arrows move the active endpoint
/// (Ctrl by a tenth of a step, Shift by ten), Home and End send it to the
/// limits, and assistive technology selects an endpoint (or the interval
/// body) and sets, increments or decrements it. A press on an endpoint or on
/// the [`movable`](Range::movable) body drags it; a press on the track drags
/// the nearer endpoint (the range scalar's track, thumbs and body).
/// The keys and the root's actions edit the endpoint (or the body) pressed,
/// selected or moved last. Each key press, action and drag is one edit
/// ([`on_edit`](Range::on_edit)); Escape and a canceled pointer during a drag
/// write both endpoints from before the press back.
#[track_caller]
pub fn range(label: impl IntoText, value: impl Into<Binding<(f64, f64)>>) -> Range {
    let label = label.into_text_value();
    Range {
        root: node()
            .role(Role::Group)
            .name(label)
            .focusable(true)
            .style(style().width(pct(100)).height(24).cursor(Cursor::EwResize)),
        label,
        state: value.into(),
        min: 0.0,
        max: 240.0,
        step: 1.0,
        interval: None,
        movable: false.into_value(),
        on_edit: None,
    }
}
impl Range {
    /// Present the interval as a flat colored block with edge handles.
    pub fn interval(mut self, color: impl IntoValue<kivact::style::Color>) -> Self {
        self.interval = Some(color.into_value());
        self
    }
    /// Drag the interval body without changing its duration or crossing the
    /// bounds while true (constant or reactive).
    #[track_caller]
    pub fn movable(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.movable = enabled.into_value();
        self
    }
    /// Report the start and end of each edit as an [`EditPhase`]: a drag of
    /// an endpoint or of the body from the press to the release, or one key
    /// press or action.
    pub fn on_edit<R: HandlerResult>(mut self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.on_edit = Some(callback(f));
        self
    }
    /// Inclusive finite bounds; invalid configurations keep the default bounds.
    pub fn limits(mut self, min: f64, max: f64) -> Self {
        if min.is_finite() && max.is_finite() && max > min {
            self.min = min;
            self.max = max;
        }
        self
    }
    pub fn step(mut self, step: f64) -> Self {
        if step.is_finite() && step > 0.0 {
            self.step = step;
        }
        self
    }
}
impl NodeBuilder for Range {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Range {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            state,
            min,
            max,
            step,
            interval,
            movable,
            on_edit,
        } = self;
        let interval_template = message("creative.range.interval", "{label} interval");
        let start_template = message("creative.range.start", "{label} start");
        let end_template = message("creative.range.end", "{label} end");
        let value = state.into_value();
        // What the root's keys and actions edit.
        let active = signal(RangePart::Start);
        let range = ScalarRange::new(min, max).expect("invariant: finite endpoint bounds");
        let bounded = move |v: (f64, f64)| {
            let a = if v.0.is_finite() {
                range.normalize(v.0).expect("invariant: finite endpoint")
            } else {
                min
            };
            let b = if v.1.is_finite() {
                v.1.clamp(a, max)
            } else {
                max
            };
            (a, b)
        };
        // One end of the value. The range scalar stops each end at the
        // other, so neither crosses nor hides the other.
        let end = |side: usize| {
            let shown = memo(move || {
                let (a, b) = bounded(value.get());
                if side == 0 { a } else { b }
            });
            binding(shown, move |v: f64| {
                let (a, b) = bounded(value.get());
                state.set(if side == 0 { (v, b) } else { (a, v) });
            })
        };
        let ends = use_range_scalar(end(0), end(1), range)
            .key_step(step)
            .expect("invariant: positive finite range step");
        let ends = match on_edit {
            Some(observer) => ends.on_edit(move |phase| observer.call(phase)),
            None => ends,
        };
        let endpoints = [ends.end(RangeThumb::Low), ends.end(RangeThumb::High)];
        // The endpoint a part edits: the body edits the end.
        let endpoint_of = move |part: RangePart| match part {
            RangePart::Start => endpoints[0],
            RangePart::End | RangePart::Body => endpoints[1],
        };
        // Actions on the root, rather than on an endpoint, edit the active
        // endpoint; the body edits the end.
        let act = move |event: A11yEvent| {
            let scalar = endpoint_of(active.get_untracked());
            match &event.request {
                A11yRequest::SetValue(text) => {
                    if let Some(v) = text.parse::<f64>().ok().filter(|v| v.is_finite()) {
                        ends.once(|| scalar.set(v));
                    }
                }
                A11yRequest::Increment => {
                    ends.once(|| scalar.step_by(1.0));
                }
                A11yRequest::Decrement => {
                    ends.once(|| scalar.step_by(-1.0));
                }
                _ => {}
            }
        };
        // The track, the thumbs and the body share the range scalar's drag:
        // a press on a thumb or on the body drags it, a press elsewhere on
        // the track drags the nearer end, and each drag is one edit of both
        // ends. The keys then edit what was pressed or moved last.
        let track = ends.track(Orientation::Horizontal);
        let pressed = |state: Value<PartState>| memo(move || state.get().pressed);
        let follow_press = move |pressed: Value<bool>, part: RangePart| {
            effect(move || {
                if pressed.get() {
                    active.set(part);
                }
            });
        };
        let moved = var(None::<(f64, f64)>);
        effect(move || {
            let now = bounded(value.get());
            match moved.get() {
                Some(before) if before.0 != now.0 && before.1 == now.1 => active.set(RangePart::Start),
                Some(before) if before.0 == now.0 && before.1 != now.1 => active.set(RangePart::End),
                _ => {}
            }
            moved.set(Some(now));
        });
        use crate::editors::shared::region_parts::{self as region_parts, Kind, RegionInfo};
        let mut root = root;
        if interval.is_some() {
            let info = kivact::memo(move || {
                RegionInfo::new(
                    RangePart::Body.region(),
                    Kind::RangeBody,
                    interpolate(&interval_template.get(), &[("label", &label.get())]),
                )
            });
            let select = callback(move |_| active.set(RangePart::Body));
            let body = region_parts::attachment(Kind::RangeBody, info, select).with(ends.body());
            follow_press(pressed(body.state()), RangePart::Body);
            root = root.region(RangePart::Body.region(), body);
        }
        for part in [RangePart::Start, RangePart::End] {
            let endpoint = endpoint_of(part);
            let template = if part == RangePart::Start {
                start_template
            } else {
                end_template
            };
            // The root keeps the one keyboard focus of the control; an
            // endpoint is reached by assistive technology and the pointer.
            let control = endpoint
                .thumb()
                .focusable(false)
                .name(move || interpolate(&template.get(), &[("label", &label.get())]))
                .on_a11y(A11yAction::Select, move || active.set(part));
            let state = control.state();
            follow_press(memo(move || state.get().input.pressed), part);
            root = root.region(part.region(), control);
        }
        root.part(track)
            .value_text(move || format!("{} – {}", value.get().0, value.get().1))
            .draw(move |d, s| {
                let p = d.theme().get::<crate::tokens::Tokens>();
                let (a, b) = bounded(value.get());
                let y = s.height / 2.0;
                // The center of an end's 20 px region, where the track puts
                // the value: the track's length less one region.
                let x =
                    |v| THUMB / 2.0 + ((v - min) / (max - min)) as f32 * (s.width - THUMB).max(0.0);
                if let Some(color) = interval {
                    let r = Rect::new(x(a), y - 7.0, (x(b) - x(a)).max(1.0), 14.0);
                    d.rounded_rect(r, 3.0, color.get().with_alpha(0.35));
                    d.stroke_rect(r, 1.0, color.get().with_alpha(0.7));
                    if movable.get() {
                        d.region(RangePart::Body.region(), r);
                    }
                } else {
                    d.rounded_rect(
                        Rect::new(8.0, y - 2.0, (s.width - 16.0).max(0.0), 4.0),
                        2.0,
                        p.strong,
                    );
                    d.rounded_rect(
                        Rect::new(x(a), y - 2.0, (x(b) - x(a)).max(0.0), 4.0),
                        2.0,
                        p.accent,
                    );
                }
                for (part, v) in [(RangePart::Start, a), (RangePart::End, b)] {
                    if interval.is_none() {
                        d.rounded_rect(
                            Rect::new(x(v) - 8.0, y - 8.0, 16.0, 16.0),
                            8.0,
                            p.text_high,
                        );
                    }
                    d.region(
                        part.region(),
                        Rect::new(x(v) - THUMB / 2.0, y - THUMB / 2.0, THUMB, THUMB),
                    );
                }
            })
            .on_a11y_event(A11yAction::Select, |_| {})
            .on_a11y_event(A11yAction::SetValue, act)
            .on_a11y_event(A11yAction::Increment, act)
            .on_a11y_event(A11yAction::Decrement, act)
            .on_key_default(move |e| {
                let part = active.get();
                let factor = if e.modifiers.ctrl {
                    0.1
                } else if e.modifiers.shift {
                    10.0
                } else {
                    1.0
                };
                if part == RangePart::Body && movable.get() {
                    let v = bounded(value.get());
                    let delta = match e.key {
                        Key::ArrowLeft | Key::ArrowDown => -step * factor,
                        Key::ArrowRight | Key::ArrowUp => step * factor,
                        Key::Home => min - v.0,
                        Key::End => max - v.1,
                        _ => return Handled::No,
                    }
                    .clamp(min - v.0, max - v.1);
                    ends.once(|| {
                        state.set((v.0 + delta, v.1 + delta));
                        true
                    });
                    return Handled::Yes;
                }
                let scalar = endpoint_of(part);
                match e.key {
                    Key::ArrowLeft | Key::ArrowDown => ends.once(|| scalar.step_by(-factor)),
                    Key::ArrowRight | Key::ArrowUp => ends.once(|| scalar.step_by(factor)),
                    Key::Home => ends.once(|| scalar.set(min)),
                    Key::End => ends.once(|| scalar.set(max)),
                    _ => return Handled::No,
                };
                Handled::Yes
            })
            .into_view()
    }
}
