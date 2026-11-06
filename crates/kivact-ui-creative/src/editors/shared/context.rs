//! Role: Shared domain snapshots for editor pointer, key and assistive menus.
use crate::editors::shared::model::EditorContext;
use crate::editors::shared::targets::Scene;
use kivact::draw::RegionId;
use kivact::state::Signal;
use kivact::view::{Handled, Key, KeyEvent, MouseButton, PointerEvent, PointerPhase};
use std::hash::Hash;

pub(crate) fn context_pointer<T: Copy>(
    event: PointerEvent,
    dragging: bool,
    target: Option<T>,
    context: Signal<EditorContext>,
    locate: impl Fn(Option<T>) -> EditorContext,
    time: impl Fn(f64) -> f64,
) -> Handled {
    if event.phase().press().map(|press| press.button) == Some(MouseButton::Secondary) {
        if dragging {
            return Handled::Yes;
        }
        if matches!(event.phase(), PointerPhase::Down(_)) {
            let mut snapshot = locate(target);
            if target.is_none() {
                snapshot.time = time(event.pos().x);
            }
            context.set(snapshot);
        }
    }
    Handled::No
}
pub(crate) fn context_key(
    event: &KeyEvent,
    child_focus: bool,
    dragging: bool,
    request: impl FnOnce(Option<RegionId>),
) -> Handled {
    if child_focus {
        return Handled::No;
    }
    if event.key() == Key::ContextMenu || (event.key() == Key::F(10) && event.modifiers().shift) {
        if dragging {
            return Handled::Yes;
        }
        request(event.region());
    }
    Handled::No
}
pub(crate) fn context_request<T: Copy + Eq + Hash>(
    scene: Option<&Scene<T>>,
    region: Option<RegionId>,
    active: Option<T>,
    fallback: Option<T>,
    context: Signal<EditorContext>,
    locate: impl FnOnce(Option<T>) -> EditorContext,
) {
    let target = scene.and_then(|scene| {
        region
            .and_then(|id| scene.target(id).copied())
            .or(active)
            .filter(|target| scene.live(target))
    });
    context.set(locate(target.or(fallback)));
}
