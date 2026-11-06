//! Role: The BezierCurve builder and its plot, handles and coordinate fields.
use super::machine;

use super::super::data::CubicBezier;
use super::super::geometry::Plane;
use kivact::custom::part;
use kivact::draw::{Path, RegionId, Stroke};
use kivact::geometry::Size;
use kivact::i18n::message;
use kivact::interaction::{DragCoordinates, DragPhase, DragUpdate, use_drag};
use kivact::state::IntoBinding;
use kivact::state::IntoValue;
use kivact::state::callback;
use kivact::state::{Binding, Callback, machine, memo};
use kivact::style::{style, use_theme};
use kivact::view::EditPhase;
use kivact::view::HandlerResult;
use kivact::view::{
    A11yAction, A11yEvent, A11yRequest, Handled, IntoView, Key, KeyEvent, Role, View, node_ref,
};
use kivact::view::{Node, NodeBuilder, node};

/// Two editable control points of a [`CubicBezier`].
/// Arrow keys move X/Y by 0.01, Shift by 0.1 and Ctrl by 0.001.
/// Each slider offers Select/Activate, SetValue (`x,y` or `x y`), and X
/// Increment/Decrement. Ordinary NodeBuilder modifiers style the canvas. External invalid values are normalized for drawing and
/// editing without writing the binding until an edit. Cancel restores the
/// exact external value, including invalid fields. Names use `bezier.label`,
/// `bezier.control1` and `bezier.control2` message keys.
pub struct BezierCurve {
    root: Node,
    value: Binding<CubicBezier>,
    on_edit: Option<Callback<EditPhase>>,
}

/// Edit a cubic easing curve. Endpoint values stay fixed at (0,0) and (1,1).
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_creative::{bezier_curve, CubicBezier};
/// fn easing() -> impl IntoView {
///     bezier_curve(signal(CubicBezier::default()))
/// }
/// ```
#[track_caller]
pub fn bezier_curve(value: impl IntoBinding<CubicBezier>) -> BezierCurve {
    BezierCurve {
        root: node()
            .role(Role::Canvas)
            .name(message("creative.bezier.label", "Cubic bezier curve"))
            .style(style().size(240, 200).min_width(0).min_height(0)),
        value: value.into_binding(),
        on_edit: None,
    }
}

impl BezierCurve {
    /// Report one edit per key/AT operation and one per drag. Escape, pointer
    /// cancellation, disabling, hiding and disposal restore the press value
    /// before Cancel. Key and AT writes are suppressed during a drag.
    pub fn on_edit<R: HandlerResult>(
        mut self,
        observer: impl Fn(EditPhase) -> R + 'static,
    ) -> Self {
        self.on_edit = Some(callback(observer));
        self
    }
}

impl NodeBuilder for BezierCurve {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

fn plane(value: CubicBezier, size: Size) -> Plane {
    Plane::new(
        size,
        0.0_f64.min(value.y1).min(value.y2),
        1.0_f64.max(value.y1).max(value.y2),
    )
}

fn pair(text: &str) -> Option<(f64, f64)> {
    let mut values = text
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty());
    let x = values.next()?.parse::<f64>().ok()?;
    let y = values.next()?.parse::<f64>().ok()?;
    (values.next().is_none() && x.is_finite() && y.is_finite()).then_some((x, y))
}

impl IntoView for BezierCurve {
    fn into_view(self) -> View {
        let binding = self.value;
        let value = binding.into_value();
        let drag = use_drag::<(usize, CubicBezier, Plane)>(DragCoordinates::Local);
        let observer = self.on_edit;
        let edit = machine(
            machine::State::Idle,
            || machine::Machine,
            move |_, effect| match effect {
                machine::Effect::Restore(initial) => binding.set(initial),
                machine::Effect::StopDrag => {
                    drag.cancel();
                }
                machine::Effect::Phase(phase) => {
                    if let Some(observer) = observer {
                        observer.call(phase);
                    }
                }
            },
        );
        kivact::state::on_cleanup(move || edit.send(machine::Event::Cancel));
        let reference = node_ref();
        let rect = reference.rect();
        let apply_drag = move |update: DragUpdate<(usize, CubicBezier, Plane)>| {
            let (index, raw, plane) = update.payload;
            let base = raw.clean();
            binding.set(base.with_point(index, plane.moved(base.point(index), update.delta)));
        };
        // Each drag is one edit; Escape, an engine Cancel and the effect
        // below cancel it, which writes the value from the press back.
        let gesture = drag
            .on_start(move |update: DragUpdate<(usize, CubicBezier, Plane)>| {
                edit.send(machine::Event::Begin(update.payload.1))
            })
            .on_update(apply_drag)
            .on_end(move |update: DragUpdate<(usize, CubicBezier, Plane)>| {
                if update.phase == DragPhase::Cancel {
                    edit.send(machine::Event::Cancel);
                } else {
                    apply_drag(update);
                    edit.send(machine::Event::Commit);
                }
            });
        // A key or an action is an edit of its own; a drag owns the value.
        let controls = Controls {
            binding,
            value,
            gesture,
            edit,
            rect,
        };
        let mut root = self
            .root
            .node_ref(reference)
            .draw(move |draw, size| paint_curve(draw, size, value));
        for (index, id, key, fallback) in [
            (
                0,
                RegionId(1),
                "creative.bezier.control1",
                "Control point 1",
            ),
            (
                1,
                RegionId(2),
                "creative.bezier.control2",
                "Control point 2",
            ),
        ] {
            let attachment = control_point(controls, index, key, fallback);
            root = root.region(id, attachment);
        }
        root.into_view()
    }
}

/// Existing binding, gesture and edit authority shared by the two control regions.
#[derive(Clone, Copy)]
struct Controls {
    binding: Binding<CubicBezier>,
    value: kivact::state::Value<CubicBezier>,
    gesture: kivact::interaction::DragApi<(usize, CubicBezier, Plane)>,
    edit: kivact::state::MachineHandle<machine::Machine>,
    rect: kivact::state::Value<Option<kivact::geometry::Rect>>,
}
impl Controls {
    /// Apply one keyboard or assistive edit only while a drag does not own the value.
    fn change(self, index: usize, next: (f64, f64)) {
        if self.gesture.is_active() {
            return;
        }
        let raw = self.value.get_untracked();
        self.edit.send(machine::Event::Begin(raw));
        self.binding.set(raw.clean().with_point(index, next));
        self.edit.send(machine::Event::Commit);
    }
}
/// Paint the fixed endpoints, curve and measured control regions.
fn paint_curve(
    draw: &mut kivact::draw::Draw,
    size: Size,
    value: kivact::state::Value<CubicBezier>,
) {
    let value = value.get().clean();
    let area = plane(value, size);
    let p = use_theme().get().ext::<crate::tokens::Tokens>();
    let from = area.point(0.0, 0.0);
    let to = area.point(1.0, 1.0);
    let first = area.point(value.x1, value.y1);
    let second = area.point(value.x2, value.y2);
    draw.line(area.point(0.0, 0.0), area.point(1.0, 0.0), 1.0, p.hair);
    draw.line(area.point(0.0, 1.0), area.point(1.0, 1.0), 1.0, p.hair);
    draw.line(from, first, 1.0, p.dim);
    draw.line(to, second, 1.0, p.dim);
    let path = Path::new().move_to(from).cubic_to(first, second, to);
    draw.stroke_path(&path, &Stroke::new(2), p.accent);
    draw.circle(from, 3.0, p.text);
    draw.circle(to, 3.0, p.text);
    for (id, point) in [(RegionId(1), first), (RegionId(2), second)] {
        draw.circle(point, 5.0, p.accent);
        draw.region(id, area.hit(point));
    }
}
/// Attach one control point's canonical input and drag part.
fn control_point(
    c: Controls,
    index: usize,
    key: &'static str,
    fallback: &'static str,
) -> kivact::custom::Part<kivact::custom::PartState> {
    let attachment = {
        let input_part = part(|input| input);
        let label = message(key, fallback);
        kivact::custom::Part::new(input_part.state(), move |node| {
            node.part(input_part)
                .role(Role::Slider)
                .name(label)
                .focusable(true)
                .value_text(move || {
                    let (x, y) = c.value.get().clean().point(index);
                    format!("{x},{y}")
                })
                .value_min(0.0)
                .value_max(1.0)
                .value_now(move || c.value.get().clean().point(index).0)
        })
    };
    let state = attachment.state();
    let target = kivact::view::node_ref();
    let attachment = kivact::custom::Part::new(attachment.state(), move |node| {
        node.part(attachment).node_ref(target)
    });
    c.edit.feed(
        memo(move || {
            let input = state.get();
            input.disabled || input.hidden
        }),
        machine::Event::InputChanged,
    );
    let action = move |event| point_action(c, index, target, event);
    let drag_source = c.gesture.source_with(move || {
        let raw = c.value.get_untracked();
        let size = c
            .rect
            .get_untracked()
            .map_or(Size::default(), |r| Size::new(r.width, r.height));
        (index, raw, plane(raw.clean(), size))
    });
    kivact::custom::Part::new(attachment.state(), move |node| {
        node.part(attachment)
            .part(drag_source)
            .on_key(move |event| point_key(c, index, event))
            .a11y_action(A11yAction::Select, action)
            .a11y_action(A11yAction::Activate, action)
            .a11y_action(A11yAction::SetValue, action)
            .a11y_action(A11yAction::Increment, action)
            .a11y_action(A11yAction::Decrement, action)
    })
}
/// Apply coordinate actions through the same single-edit transaction boundary.
fn point_action(c: Controls, index: usize, target: kivact::view::NodeRef, event: A11yEvent) {
    if matches!(event.request, A11yRequest::Select | A11yRequest::Activate) {
        target.focus();
        return;
    }
    if c.gesture.is_active() {
        return;
    }
    let (x, y) = c.value.get_untracked().clean().point(index);
    let next = match &event.request {
        A11yRequest::SetValue(text) => pair(text),
        A11yRequest::Increment => Some((x + 0.01, y)),
        A11yRequest::Decrement => Some((x - 0.01, y)),
        _ => None,
    };
    if let Some(next) = next {
        c.change(index, next);
    }
}
/// Resolve modified arrow steps without changing drag ownership.
fn point_key(c: Controls, index: usize, event: KeyEvent) -> Handled {
    let step = if event.modifiers().ctrl {
        0.001
    } else if event.modifiers().shift {
        0.1
    } else {
        0.01
    };
    let (x, y) = c.value.get().clean().point(index);
    let next = match event.key() {
        Key::ArrowLeft => (x - step, y),
        Key::ArrowRight => (x + step, y),
        Key::ArrowUp => (x, y + step),
        Key::ArrowDown => (x, y - step),
        _ => return Handled::No,
    };
    c.change(index, next);
    Handled::Yes
}
