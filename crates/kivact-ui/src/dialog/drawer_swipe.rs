//! Role: Attach velocity-aware dismissal to a drawer's dedicated drag handle.
use crate::dialog::look::DialogTheme;
use kivact_core::geometry::Edge;
use kivact_core::state::{Signal, Value, memo};
use kivact_core::style::{style, use_theme};
use kivact_core::view::{Node, NodeRef, node};
use kivact_interaction::{InteractionPolicy, SwipeDismissPolicy, SwipePhase, use_swipe_dismiss};

pub(crate) fn handle(
    edge: Edge,
    panel: NodeRef,
    open: Signal<bool>,
    variants: &kivact_core::view::Variants,
) -> (Node, Value<f64>) {
    let policy = use_theme().get().ext::<InteractionPolicy>().drawer.swipe;
    let swipe = use_swipe_dismiss(
        SwipeDismissPolicy::drawer(edge, policy),
        panel.rect(),
        move || open.set(false),
    );
    let part = swipe.control();
    let state = part.state();
    let phase = swipe.phase();
    let pulled = memo(move || match phase.get() {
        SwipePhase::Dragging { offset, .. } => offset,
        SwipePhase::Dismissed { travel, .. } => travel,
        SwipePhase::Rest | SwipePhase::Pressed => 0.0,
    });
    let handle =
        crate::layer::with_recipe_style(
            node().part(part).role(kivact_core::view::Role::Group).name(
                kivact_core::i18n::message("drawer.drag_handle", "Drag to close"),
            ),
            |r: &DialogTheme| r.drag_handle_recipe(),
            variants,
            move || crate::theme::DialogState {
                input: state.get(),
                open: open.get(),
                kind: crate::DialogKind::Drawer(edge),
                media: false,
            },
        );
    (handle, pulled)
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
