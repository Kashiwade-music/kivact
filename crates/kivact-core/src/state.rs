//! Role: Reactive handles, setup, contexts and state machines.

pub use crate::view::boundary::provide;
pub use crate::view::owner::{Owner, use_app_state};
pub use kivact_runtime::{
    Callback, IntoHandler, IntoText, IntoValue, ItemKey, Machine, MachineCx, MachineHandle,
    PropValue, Selector, Signal, Step, Trigger, Value, Var, bind, binding, callback,
    constant_value, effect, effect_on, machine, memo, on_cleanup, on_mount, selector, signal,
    trigger, try_use_context, untrack, use_context, var,
};

#[doc(hidden)]
pub mod __private;
