//! Role: Report whether the current owner's view has been committed.

use kivact_runtime::{Value, on_mount, signal};

/// Whether the view the current setup creates is live (tracked): `false`
/// during setup, `true` from the mount pass in which the host has applied
/// this setup's nodes and the owner's `on_mount` callbacks run (this one
/// runs in its registration order among them), until the owner is
/// disposed. Setup only.
///
/// ```no_run
/// use kivact_core::state::effect;
/// use kivact_core::view::is_mounted;
/// // During component setup:
/// let mounted = is_mounted();
/// effect(move || {
///     if mounted.get() {
///         // The nodes of this setup are live.
///     }
/// });
/// ```
#[track_caller]
pub fn is_mounted() -> Value<bool> {
    let mounted = signal(false);
    on_mount(move || mounted.set(true));
    mounted.into()
}
