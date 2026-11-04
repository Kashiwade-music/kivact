//! Role: Reactive handles, setup, contexts and state machines.

pub use crate::view::boundary::provide;
pub use crate::view::owner::{Owner, use_app_state};
pub use kivact_runtime::{
    Binding, Callback, IntoBinding, IntoReactive, IntoText, IntoValue, ItemKey, Machine, MachineCx,
    MachineHandle, Notifier, PropValue, Selector, Signal, Step, Value, Var, bind, binding,
    callback, constant_value, effect, effect_on, machine, memo, notifier, on_cleanup, on_mount,
    selector, signal, try_use_context, untrack, use_context, var,
};

// xtask-allow(internal-contract): Macro expansion support only; users consume IntoValue, not its representation.
#[doc(hidden)]
pub mod __private;
