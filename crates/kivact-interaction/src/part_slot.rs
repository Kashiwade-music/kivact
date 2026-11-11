//! Role: The slot through which a handle reads the state of its attached part.
use kivact_core::state::{Signal, Value, on_cleanup};

/// Put the state of a part being made in `slot`, and empty the slot when the
/// owner making it ends.
///
/// The state is owned by that owner (the trigger's or the content's mount,
/// often a `show` branch), so the slot holds it exactly while it lives: a
/// handle that reads the slot then falls back to its own state after the
/// part unmounts, and reads the new part after it mounts again.
pub(crate) fn attach_part_state<S: 'static>(slot: Signal<Option<Value<S>>>, state: Value<S>) {
    slot.set(Some(state));
    on_cleanup(move || {
        // The handle's own owner may have ended first, with the slot.
        if slot.try_get_untracked().is_some() {
            slot.update(|attached| {
                if *attached == Some(state) {
                    *attached = None;
                }
            });
        }
    });
}
