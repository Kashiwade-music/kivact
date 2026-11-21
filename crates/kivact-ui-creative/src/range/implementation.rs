//! Role: Two-endpoint range control of the Creative kit.

use super::data::RangePart;
use super::machine;
use crate::color::{ACCENT, STRONG, TEXT_HIGH};
use kivact::component;
use kivact::custom::{Caller, PartState};
use kivact::geometry::Rect;
use kivact::i18n::{interpolate, message};
use kivact::interaction::ScalarRange;
use kivact::interaction::ScalarStep;
use kivact::interaction::{RangeThumb, use_range_scalar};
use kivact::state::Signal;
use kivact::state::{Callback, Value, binding, machine, memo};
use kivact::state::{IntoValue, callback};
use kivact::style::{Cursor, pct, style};
use kivact::view::EditPhase;
use kivact::view::{A11yAction, A11yEvent, A11yRequest, Handled, Key, Orientation, Role};
use kivact::view::{Node, node};

/// The side of an end's square region, which the pointer grabs.
const THUMB: f64 = 20.0;

/// Two endpoints of `value` on one track, start before end, edited without
/// crossing or hiding the other endpoint. `label` names the control (the
/// element's accessible name); `limits` sets the inclusive bounds and `step`
/// the keyboard and wheel increment (invalid configurations keep the
/// defaults, 0 to 240 and 1). `interval` presents the interval as a flat
/// colored block with edge handles, and `movable` drags the interval body
/// without changing its duration or crossing the bounds while true.
/// `on_edit` reports the start and end of each edit as an [`EditPhase`]: a
/// drag of an endpoint or of the body from the press to the release, or one
/// key press or action.
///
/// The control keeps one keyboard focus: the arrows move the active endpoint
/// (Ctrl by a tenth of a step, Shift by ten), Home and End send it to the
/// limits, and assistive technology selects an endpoint (or the interval
/// body) and sets, increments or decrements it. A press on an endpoint or on
/// the movable body drags it; a press on the track drags the nearer endpoint
/// (the range scalar's track, thumbs and body). The keys and the root's
/// actions edit the endpoint (or the body) pressed, selected or moved last.
/// Each key press, action and drag is one edit; Escape and a canceled pointer
/// during a drag write both endpoints from before the press back.
#[component]
pub fn range(
    value: Signal<(f64, f64)>,
    interval: Option<Value<kivact::style::Color>>,
    #[prop(default = false)] movable: Value<bool>,
    on_edit: Option<Callback<EditPhase>>,
    limits: Option<Value<ScalarRange>>,
    step: Option<Value<f64>>,
    #[caller] caller: Caller,
) -> impl Element {
    let label = caller.name().unwrap_or_else(|| String::new().into_value());
    let root = node()
        .role(Role::Group)
        .name(label)
        .focusable(true)
        .style(style().width(pct(100)).height(24).cursor(Cursor::EwResize));
    let (min, max) = match limits {
        Some(range) => (
            memo(move || range.get().min()),
            memo(move || range.get().max()),
        ),
        None => (0.0.into_value(), 240.0.into_value()),
    };
    let step = match step {
        Some(step) => memo(move || {
            let step = step.get();
            if step.is_finite() && step > 0.0 {
                step
            } else {
                1.0
            }
        }),
        None => 1.0.into_value(),
    };
    let state = value;
    let value = state.into_value();
    let range = memo(move || {
        ScalarRange::new(min.get(), max.get()).expect("invariant: finite endpoint bounds")
    });
    let bounded = move |v| bounded_pair(v, range, min, max);
    let controller = machine(
        machine::State {
            active: RangePart::Start,
            value: bounded(value.get_untracked()),
        },
        || machine::Machine,
        |_, ()| {},
    );
    let active = memo(move || controller.get().active);
    controller.feed(memo(move || bounded(value.get())), move |now| {
        machine::Event::ValueChanged {
            before: controller.state().get_untracked().value,
            now,
        }
    });
    let ends = range_ends(state, range, min, max, step, on_edit);
    let act = move |event| range_action(event, active, ends);
    // The track, the thumbs and the body share the range scalar's drag:
    // a press on a thumb or on the body drags it, a press elsewhere on
    // the track drags the nearer end, and each drag is one edit of both
    // ends. The keys then edit what was pressed or moved last.
    let track = ends.track(Orientation::Horizontal);
    let root = range_regions(root, label, interval.is_some(), ends, controller);
    root.part(track)
        .value_text(move || format!("{} – {}", value.get().0, value.get().1))
        .draw(move |d, s| {
            paint_range(
                d,
                s,
                move || bounded(value.get()),
                min,
                max,
                interval,
                movable,
            )
        })
        .a11y_action(A11yAction::Select, |_| {})
        .a11y_action(A11yAction::SetValue, act)
        .a11y_action(A11yAction::Increment, act)
        .a11y_action(A11yAction::Decrement, act)
        .on_key_default(move |e| {
            range_key(
                e,
                active,
                movable,
                move || bounded(value.get()),
                state,
                ends,
                (min, max, step),
            )
        })
}

fn bounded_pair(
    v: (f64, f64),
    range: Value<ScalarRange>,
    min: Value<f64>,
    max: Value<f64>,
) -> (f64, f64) {
    let a = if v.0.is_finite() {
        range
            .get()
            .normalize(v.0)
            .expect("invariant: finite endpoint")
    } else {
        min.get()
    };
    let b = if v.1.is_finite() {
        v.1.clamp(a, max.get())
    } else {
        max.get()
    };
    (a, b)
}

fn paint_range(
    d: &mut kivact::draw::Draw,
    s: kivact::geometry::Size,
    pair: impl Fn() -> (f64, f64),
    min: Value<f64>,
    max: Value<f64>,
    interval: Option<Value<kivact::style::Color>>,
    movable: Value<bool>,
) {
    let (a, b) = pair();
    let y = s.height / 2.0;
    // The center of an end's 20 px region, where the track puts
    // the value: the track's length less one region.
    let x =
        |v| THUMB / 2.0 + ((v - min.get()) / (max.get() - min.get())) * (s.width - THUMB).max(0.0);
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
            STRONG,
        );
        d.rounded_rect(
            Rect::new(x(a), y - 2.0, (x(b) - x(a)).max(0.0), 4.0),
            2.0,
            ACCENT,
        );
    }
    for (part, v) in [(RangePart::Start, a), (RangePart::End, b)] {
        if interval.is_none() {
            d.rounded_rect(Rect::new(x(v) - 8.0, y - 8.0, 16.0, 16.0), 8.0, TEXT_HIGH);
        }
        d.region(
            part.region(),
            Rect::new(x(v) - THUMB / 2.0, y - THUMB / 2.0, THUMB, THUMB),
        );
    }
}

fn range_key(
    e: kivact::view::KeyEvent,
    active: Value<RangePart>,
    movable: Value<bool>,
    pair: impl Fn() -> (f64, f64),
    state: Signal<(f64, f64)>,
    ends: kivact::interaction::RangeScalarApi,
    bounds: (Value<f64>, Value<f64>, Value<f64>),
) -> Handled {
    let (min, max, step) = bounds;
    let part = active.get();
    let factor = if e.modifiers().ctrl {
        0.1
    } else if e.modifiers().shift {
        10.0
    } else {
        1.0
    };
    if part == RangePart::Body && movable.get() {
        let v = pair();
        let delta = match e.key() {
            Key::ArrowLeft | Key::ArrowDown => -step.get() * factor,
            Key::ArrowRight | Key::ArrowUp => step.get() * factor,
            Key::Home => min.get() - v.0,
            Key::End => max.get() - v.1,
            _ => return Handled::No,
        }
        .clamp(min.get() - v.0, max.get() - v.1);
        ends.once(|| {
            state.set((v.0 + delta, v.1 + delta));
            true
        });
        return Handled::Yes;
    }
    let scalar = ends.end(if part == RangePart::Start {
        RangeThumb::Low
    } else {
        RangeThumb::High
    });
    match e.key() {
        Key::ArrowLeft | Key::ArrowDown => ends.once(|| scalar.step_by(-factor)),
        Key::ArrowRight | Key::ArrowUp => ends.once(|| scalar.step_by(factor)),
        Key::Home => ends.once(|| scalar.set(min.get())),
        Key::End => ends.once(|| scalar.set(max.get())),
        _ => return Handled::No,
    };
    Handled::Yes
}

fn range_action(
    event: A11yEvent,
    active: Value<RangePart>,
    ends: kivact::interaction::RangeScalarApi,
) {
    let scalar = ends.end(if active.get_untracked() == RangePart::Start {
        RangeThumb::Low
    } else {
        RangeThumb::High
    });
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
}

fn range_regions(
    mut root: Node,
    label: Value<String>,
    interval: bool,
    ends: kivact::interaction::RangeScalarApi,
    controller: kivact::state::MachineHandle<machine::Machine>,
) -> Node {
    let end_template = message("creative.range.end", "{label} end");
    let start_template = message("creative.range.start", "{label} start");
    let interval_template = message("creative.range.interval", "{label} interval");
    let pressed = |state: Value<PartState>| memo(move || state.get().is_pressed());
    let follow_press = move |pressed: Value<bool>, part: RangePart| {
        controller.feed(pressed, move |pressed| machine::Event::Press {
            part,
            pressed,
        });
    };
    use crate::editors::shared::region_parts::{self as region_parts, Kind, RegionInfo};

    if interval {
        let info = kivact::state::memo(move || {
            RegionInfo::new(
                RangePart::Body.region(),
                Kind::RangeBody,
                interpolate(&interval_template.get(), &[("label", &label.get())]),
            )
        });
        let select = callback(move |_| controller.send(machine::Event::Select(RangePart::Body)));
        let body = region_parts::attachment(Kind::RangeBody, info, select);
        let body =
            kivact::custom::Part::new(body.state(), move |node| node.part(body).part(ends.body()));
        follow_press(pressed(body.state()), RangePart::Body);
        root = root.region(RangePart::Body.region(), body);
    }
    for part in [RangePart::Start, RangePart::End] {
        let endpoint = ends.end(if part == RangePart::Start {
            RangeThumb::Low
        } else {
            RangeThumb::High
        });
        let template = if part == RangePart::Start {
            start_template
        } else {
            end_template
        };
        // The root keeps the one keyboard focus of the control; an
        // endpoint is reached by assistive technology and the pointer.
        let control = endpoint.thumb();
        let control = kivact::custom::Part::new(control.state(), move |node| {
            node.part(control)
                .focusable(false)
                .name(move || interpolate(&template.get(), &[("label", &label.get())]))
                .a11y_action(A11yAction::Select, move |_| {
                    controller.send(machine::Event::Select(part))
                })
        });
        let state = control.state();
        follow_press(memo(move || state.get().input.is_pressed()), part);
        root = root.region(part.region(), control);
    }
    root
}

fn range_ends(
    state: Signal<(f64, f64)>,
    range: Value<ScalarRange>,
    min: Value<f64>,
    max: Value<f64>,
    step: Value<f64>,
    on_edit: Option<Callback<EditPhase>>,
) -> kivact::interaction::RangeScalarApi {
    let value = state.into_value();
    let bounded = move |v| bounded_pair(v, range, min, max);
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
        .key_step(memo(move || ScalarStep::Value(step.get())))
        .expect("invariant: positive finite range step");
    match on_edit {
        Some(observer) => ends.on_edit(move |phase| observer.call(phase)),
        None => ends,
    }
}
