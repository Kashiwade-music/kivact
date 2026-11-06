//! Role: Observe the existing input-group projection from deferred descendant scopes.
use crate::component_values::InputGroupState;
use kivact_core::state::{Value, provide, try_use_context};
use kivact_core::view::{IntoView, View};
#[derive(Clone, Copy)]
struct InputGroupContext(Value<InputGroupState>);
/// Observe the closest mounted group's actual input, focus and layout facts.
/// This read-only value has no group controller or independent state owner.
pub fn use_input_group_state() -> Option<Value<InputGroupState>> {
    try_use_context::<InputGroupContext>().map(|context| context.get().0)
}
pub(crate) fn provide_state<V: IntoView>(
    state: Value<InputGroupState>,
    content: impl FnOnce() -> V + 'static,
) -> View {
    provide(InputGroupContext(state), content)
}
