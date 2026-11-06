//! Role: Attach velocity-aware dismissal to a drawer's dedicated drag handle.
use kivact_core::geometry::{Edge, Point};
use kivact_core::state::{Binding, Value, signal};
use kivact_core::style::{style, use_theme};
use kivact_core::view::{Node, NodeRef, node};
use kivact_interaction::{DragCoordinates, DragPhase, DragUpdate, InteractionPolicy, use_drag};

pub(crate) fn handle(
    edge: Edge,
    panel: NodeRef,
    open: Binding<bool>,
    variants: &crate::variants::Variants,
) -> (Node, Value<f64>) {
    let pulled = signal(0.0);
    let policy = use_theme().get().ext::<InteractionPolicy>().drawer.swipe;
    let rect = panel.rect();
    let drag = use_drag::<()>(DragCoordinates::Window)
        .track_velocity()
        .on_update(move |update: DragUpdate<()>| {
            pulled.set(if update.phase == DragPhase::Cancel {
                0.0
            } else {
                toward(edge, update.delta).max(0.0)
            });
        })
        .on_end(move |update: DragUpdate<()>| {
            let distance = toward(edge, update.delta).max(0.0);
            let far = rect.get().is_some_and(|rect| {
                let size = if matches!(edge, Edge::Left | Edge::Right) {
                    rect.width
                } else {
                    rect.height
                };
                size > 0.0 && distance > size * policy.close_fraction
            });
            let flick = distance > policy.flick_distance
                && toward(edge, update.velocity) > policy.flick_speed;
            if update.phase == DragPhase::End && (far || flick) {
                open.set(false);
            }
            pulled.set(0.0);
        });
    let part = drag.source(());
    let state = part.state();
    let handle =
        crate::layer::with_recipe_style(
            node().part(part).role(kivact_core::view::Role::Group).name(
                kivact_core::i18n::message("drawer.drag_handle", "Drag to close"),
            ),
            |recipes| recipes.dialog_recipes().drag_handle_recipe(),
            variants,
            move || crate::DialogState {
                input: state.get().input,
                open: open.get(),
                kind: kivact_interaction::DialogMode::Drawer(edge),
                media: false,
            },
        );
    (handle, pulled.into())
}
pub(crate) fn translation(edge: Edge, distance: f64) -> kivact_core::style::Style {
    let (x, y) = match edge {
        Edge::Left => (-distance, 0.0),
        Edge::Right => (distance, 0.0),
        Edge::Top => (0.0, -distance),
        Edge::Bottom => (0.0, distance),
    };
    style().translate(x, y)
}
fn toward(edge: Edge, point: Point) -> f64 {
    match edge {
        Edge::Left => -point.x,
        Edge::Right => point.x,
        Edge::Top => -point.y,
        Edge::Bottom => point.y,
    }
}
