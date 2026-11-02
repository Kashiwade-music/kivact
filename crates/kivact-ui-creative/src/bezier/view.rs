//! Role: The BezierCurve builder and its plot, handles and coordinate fields.

use super::data::CubicBezier;
use super::geometry::Plane;
use kivact::control::{Node, NodeBuilder, node};
use kivact::draw::{Path, RegionId, Size, Stroke};
use kivact::i18n::message;
use kivact::interaction::{DragCoordinates, DragPhase, DragUpdate, EditSession, use_drag};
use kivact::primitive::part;
use kivact::style::{current_theme, style};
use kivact::ui::EditPhase;
use kivact::view::HandlerResult;
use kivact::view::{
    A11yAction, A11yEvent, A11yRequest, Handled, IntoValue, IntoView, Key, KeyEvent, Role, View,
    callback, node_ref,
};
use kivact::{Binding, Callback, effect, var};

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
pub fn bezier_curve(value: impl Into<Binding<CubicBezier>>) -> BezierCurve {
    BezierCurve {
        root: node()
            .role(Role::Canvas)
            .name(message("creative.bezier.label", "Cubic bezier curve"))
            .style(style().size(240, 200).min_width(0).min_height(0)),
        value: value.into(),
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
        let edit = crate::editors::shared::model::observed(
            EditSession::new(move |original| binding.set(original)),
            self.on_edit,
        );
        let reference = node_ref();
        let rect = reference.rect();
        let snapshot = var((value.get_untracked(), Plane::new(Size::default(), 0.0, 1.0)));
        let apply_drag = move |update: DragUpdate<usize>| {
            let (raw, plane) = snapshot.get();
            let base = raw.clean();
            let index = update.payload;
            binding.set(base.with_point(index, plane.moved(base.point(index), update.delta)));
        };
        // Each drag is one edit; Escape, an engine Cancel and the effect
        // below cancel it, which writes the value from the press back.
        let gesture = use_drag::<usize>(DragCoordinates::Local)
            .edit(edit, move || value.get_untracked())
            .on_start(move |_update: DragUpdate<usize>| {
                let raw = value.get();
                let clean = raw.clean();
                let size = rect
                    .get()
                    .map_or(Size::default(), |r| Size::new(r.width, r.height));
                snapshot.set((raw, plane(clean, size)));
            })
            .on_update(apply_drag)
            .on_end(move |update: DragUpdate<usize>| {
                if update.phase != DragPhase::Cancel {
                    apply_drag(update);
                }
            });
        // A key or an action is an edit of its own; a drag owns the value.
        let change = move |index: usize, next: (f64, f64)| {
            if gesture.is_active() {
                return;
            }
            let raw = value.get_untracked();
            edit.once(raw, || binding.set(raw.clean().with_point(index, next)));
        };
        let mut root = self.root.node_ref(reference).draw(move |draw, size| {
            let value = value.get().clean();
            let area = plane(value, size);
            let p = current_theme().get::<crate::tokens::Tokens>();
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
        });
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
            let attachment = part(|input| input)
                .role(Role::Slider)
                .name(message(key, fallback))
                .focusable(true)
                .value_text(move || {
                    let (x, y) = value.get().clean().point(index);
                    format!("{x},{y}")
                })
                .value_min(0.0)
                .value_max(1.0)
                .value_now(move || value.get().clean().point(index).0);
            let state = attachment.state();
            let target = attachment.node_ref();
            effect(move || {
                let state = state.get();
                if state.disabled || state.hidden {
                    gesture.cancel();
                }
            });
            let action = move |event: A11yEvent| {
                if matches!(event.request, A11yRequest::Select | A11yRequest::Activate) {
                    target.focus();
                    return;
                }
                if gesture.is_active() {
                    return;
                }
                let (x, y) = value.get_untracked().clean().point(index);
                let next = match &event.request {
                    A11yRequest::SetValue(text) => pair(text),
                    A11yRequest::Increment => Some((x + 0.01, y)),
                    A11yRequest::Decrement => Some((x - 0.01, y)),
                    _ => None,
                };
                if let Some(next) = next {
                    change(index, next);
                }
            };
            let attachment = attachment
                .with(gesture.source(index))
                .on_key(move |event: KeyEvent| {
                    let step = if event.modifiers.ctrl {
                        0.001
                    } else if event.modifiers.shift {
                        0.1
                    } else {
                        0.01
                    };
                    let (x, y) = value.get().clean().point(index);
                    let next = match event.key {
                        Key::ArrowLeft => (x - step, y),
                        Key::ArrowRight => (x + step, y),
                        Key::ArrowUp => (x, y + step),
                        Key::ArrowDown => (x, y - step),
                        _ => return Handled::No,
                    };
                    change(index, next);
                    Handled::Yes
                })
                .on_a11y_event(A11yAction::Select, action)
                .on_a11y_event(A11yAction::Activate, action)
                .on_a11y_event(A11yAction::SetValue, action)
                .on_a11y_event(A11yAction::Increment, action)
                .on_a11y_event(A11yAction::Decrement, action);
            root = root.region(id, attachment);
        }
        root.into_view()
    }
}
