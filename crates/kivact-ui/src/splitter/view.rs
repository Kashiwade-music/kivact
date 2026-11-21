//! Role: The splitter component, its two panes and the handle between them.

use crate::control_size::ControlSize;
use crate::layer::with_recipe_style;
use crate::slider::look::AxisState;
use crate::splitter::look::SplitterTheme;
use kivact_core::ErrorKind;
use kivact_core::custom::{Caller, Owner};
use kivact_core::state::{Callback, Signal, Value};
use kivact_core::style::{FlexDirection, style};
use kivact_core::view::{EditPhase, View, node, node_ref};
use kivact_core::view::{Orientation, ReadingDirection, Role, use_direction};
use kivact_interaction::{ScalarRange, SplitterApi, use_splitter};
use kivact_macros::component;

/// Two panes side by side with a themed handle between them, named `name`,
/// that edits `size`: the leading pane's width in pixels, within `range`.
///
/// The handle is a `Role::Splitter` with the value and its bounds. While it
/// is focused the arrows step the size (ArrowLeft shrinks the leading pane,
/// ArrowUp when the panes are stacked), Home and End go to the bounds, and
/// assistive technology increments, decrements and sets it. A drag moves it
/// with the pointer; Escape or a canceled pointer during the drag writes the
/// size from before the press back. Each key press, action and drag is one
/// edit (`on_edit`). In a right-to-left subtree the leading pane is on the
/// right and the keys and the drag follow it.
///
/// Options: `collapsible(threshold)` collapses on release below that fraction
/// of the current range; `snaps(stops)` snaps on release to the nearest
/// fraction; `pointer_scale(scale)` scales pointer travel into the caller's
/// size units; `key_step(step)` sets the keyboard step in the caller's size
/// units (a step that is not positive and finite is reported when the
/// splitter is built, and the default step is used); `handle(content)`
/// supplies the handle's visual content (its canonical input part stays
/// attached); `orientation(..)` is the direction the panes are laid out in
/// (default `Orientation::Horizontal`: side by side, `size` is the leading
/// pane's width; `Orientation::Vertical` stacks them, `size` is the top
/// pane's height); `start(..)` is the leading pane (left, right in a
/// right-to-left subtree, or top); `end(..)` is the trailing pane, which
/// takes the rest of the room; `on_edit(..)` reports the start and end of
/// each edit (`EditPhase`): a drag from the press to the release, or one key
/// press or action.
///
/// ```no_run
/// use kivact::interaction::ScalarRange;
/// use kivact::prelude::*;
/// use kivact::ui::splitter;
///
/// fn editor() -> impl IntoView {
///     let sidebar = signal(240.0);
///     let range = ScalarRange::new(120.0, 480.0).expect("a valid range");
///     splitter("Sidebar width", sidebar, range)
///         .start(text("Files"))
///         .end(text("Editor"))
/// }
/// ```
#[component]
pub fn splitter(
    name: Value<String>,
    size: Signal<f64>,
    range: Value<ScalarRange>,
    start: Option<View>,
    end: Option<View>,
    handle: Option<View>,
    on_edit: Option<Callback<EditPhase>>,
    collapsible: Option<f64>,
    snaps: Option<Value<Vec<f64>>>,
    #[prop(default = 1.0)] pointer_scale: Value<f64>,
    key_step: Option<f64>,
    #[prop(default = Orientation::Horizontal)] orientation: Orientation,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let vertical = orientation == Orientation::Vertical;
    let mut behavior = use_splitter(size, range, orientation).on_edit(move |phase| {
        if let Some(on_edit) = on_edit {
            on_edit.call(phase);
        }
    });
    behavior = behavior.pointer_scale(pointer_scale);
    if let Some(threshold) = collapsible {
        behavior = behavior.collapsible(threshold);
    }
    if let Some(stops) = snaps {
        behavior = behavior.snaps(stops);
    }
    behavior = with_key_step(behavior, key_step);
    // `.disabled(..)` on the splitter ignores input and assistive
    // technology and cancels a drag in progress.
    let disabled = caller.disabled();
    let part = behavior.handle();
    let input = part.state();
    let primary = node_ref();
    let handle_node = with_recipe_style(
        node()
            .disabled(move || disabled.is_some_and(|disabled| disabled.get()))
            .part(part)
            .name(name)
            .controls(primary)
            .child(handle),
        |r: &SplitterTheme| r.handle_recipe(),
        &variants,
        move || AxisState {
            input: input.get().input,
            orientation,
            size: ControlSize::Medium,
        },
    );
    let leading = node()
        .role(Role::Group)
        .node_ref(primary)
        .name(name)
        .child(start)
        .style(move || {
            let px = size.get();
            let s = style().flex_shrink(0).min_width(0).min_height(0);
            if vertical { s.height(px) } else { s.width(px) }
        });
    let trailing = node().child(end).style(
        style()
            .flex_grow(1)
            .flex_basis(0)
            .min_width(0)
            .min_height(0),
    );
    let direction = match (vertical, use_direction()) {
        (true, _) => FlexDirection::Column,
        (false, ReadingDirection::Ltr) => FlexDirection::Row,
        (false, ReadingDirection::Rtl) => FlexDirection::RowReverse,
    };
    node()
        .base_style(style().flex_direction(direction).min_width(0).min_height(0))
        .child(leading)
        .child(handle_node)
        .child(trailing)
}

/// Give the handle its keyboard step. A step that is not positive and finite
/// is reported once, here at build time, and the default step is used.
fn with_key_step(behavior: SplitterApi, step: Option<f64>) -> SplitterApi {
    match step {
        Some(step) if step.is_finite() && step > 0.0 => behavior
            .key_step(step)
            .expect("invariant: the keyboard step was checked above"),
        Some(step) => {
            Owner::current().report(
                ErrorKind::Message,
                format!(
                    "splitter key_step must be positive and finite, got {step}; the default step is used"
                ),
            );
            behavior
        }
        None => behavior,
    }
}
