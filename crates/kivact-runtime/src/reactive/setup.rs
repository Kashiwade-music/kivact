//! Role: Free setup functions for reactive state and effects.
//!
//! Free setup functions: `signal`, `memo`, `effect`, `effect_on`,
//! `on_cleanup`, `on_mount`, `use_context`, `var`, `binding`, plus the two
//! lookups every Kivact crate uses for its own constructors
//! ([`setup_scope`], [`owner_scope`]). There is no setup-context argument:
//! the executing App and the owner come from the runtime's frame stack.
//!
//! Invariants:
//! - Creation and registration need a `Setup` frame of a live owner
//!   (`Runtime::require_setup`). Anywhere else (handlers, tasks after an
//!   `.await`, effects, outside every App) they raise a located `SetupOnly`
//!   misuse. Nothing is attached to the root owner implicitly.
//! - Owner-relative lookups (`use_context`, `spawn`, `platform::window_size`,
//!   `style::use_theme`, ...) use the nearest owner of the frame stack
//!   (`Runtime::current_owner`): a handler uses the owner of the node it is
//!   delivered to, a task its own owner and `Callback::call` the callback's
//!   owner, so they also work in handlers, tasks, frame callbacks and
//!   effects. When
//!   that owner is already disposed (a queue draining after its owner went
//!   away) they raise a located `Disposed`, never `SetupOnly`.
//! - No new global state: only the existing App registry and frame stack.

use std::any::{TypeId, type_name};
use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use crate::HandlerResult;
use crate::reactive::arena::Key;
use crate::reactive::data::{
    Binding, BindingRepr, Callback, CallbackFn, EffectKind, Runtime, Signal, Value, Var,
};
use crate::reactive::error::{ErrorKind, misuse_at};
use crate::reactive::props::IntoValue;
use crate::reactive::registry::current_runtime;
use crate::reactive::scope::{owner_scope, setup_scope};

/// A writable reactive value (Solid `createSignal`). Setup only.
///
/// ```ignore
/// let count = signal(0);
/// button("+1").on_press(move || count.update(|c| *c += 1))
/// ```
#[track_caller]
pub fn signal<T: 'static>(value: T) -> Signal<T> {
    let at = Location::caller();
    let (rt, owner) = setup_scope("signal", at);
    Signal::from_key(rt.id(), rt.create_source(owner, value, at))
}

/// An untracked local variable of the current owner (like React `useRef`),
/// writable anywhere, including memos (caches). Setup only.
#[track_caller]
pub fn var<T: 'static>(value: T) -> Var<T> {
    let at = Location::caller();
    let (rt, owner) = setup_scope("var", at);
    let key = rt.insert_object(owner, Rc::new(RefCell::new(value)), "Var", None, at);
    Var::from_key(rt.id(), key)
}

/// A two-way input binding made of a value and the closure that writes it,
/// for joining two separate pieces of state (a field of one value already
/// has [`Signal::project`]; keyed lists supply row bindings). The value
/// can be anything [`IntoValue`] takes; `set` is called with each new value
/// and may return `()` or `Result<(), UiError>`. Setup only.
///
/// ```ignore
/// let celsius = signal(20.0);
/// let fahrenheit = binding(move || celsius.get() * 1.8 + 32.0, move |f| celsius.set((f - 32.0) / 1.8));
/// ```
#[track_caller]
pub fn binding<T: crate::PropValue, R: HandlerResult>(
    value: impl IntoValue<T>,
    set: impl Fn(T) -> R + 'static,
) -> Binding<T> {
    Binding {
        repr: BindingRepr::Pair(value.into_value(), callback(set)),
    }
}

/// A Copy callback of the current owner, for passing the same closure to
/// several places and to child components (child to parent). `f` is
/// `|arg| ..` (`|_| ..` for a callback without input, a `Callback<()>` called
/// with `.call(())`), and may return `()` or `Result<(), UiError>`; failures
/// go to the nearest error handler. A closure that calls a method on its
/// argument needs the argument type (`callback(|e: KeyEvent| e.key)`),
/// because rustc does not infer it through `impl Fn(E)`.
/// Wherever it is called from (setup, effect, handler, task), `spawn`,
/// `context` and the `kivact::platform` functions inside it use the
/// callback's owner. A call is always untracked: what the callback reads is
/// not a dependency of a calling effect or memo (so is an event handler, an
/// `on_mount` callback and the body of a task, an action or a resource
/// fetch). Everything else is the caller's: purity is kept, and `signal` and
/// the other setup functions work only when it is called during setup,
/// creating under the caller's setup owner. Setup only.
///
/// ```ignore
/// let on_pick = callback(move |id: u64| selected.set(Some(id)));
/// let reload = callback(|_| load());
/// ```
#[track_caller]
pub fn callback<E: 'static, R: HandlerResult>(f: impl Fn(E) -> R + 'static) -> Callback<E> {
    let at = Location::caller();
    let (rt, owner) = setup_scope("callback", at);
    let cb: Rc<CallbackFn<E>> = Rc::new(CallbackFn {
        owner,
        f: Box::new(move |e| f(e).into_outcome().map(|_| ())),
    });
    Callback::from_key(rt.id(), rt.insert_object(owner, cb, "Callback", None, at))
}

/// A cached derived value (Solid `createMemo`); readers are notified only
/// when the result changes (`PartialEq`). Setup only.
#[track_caller]
pub fn memo<T: PartialEq + 'static>(f: impl Fn() -> T + 'static) -> Value<T> {
    let at = Location::caller();
    let (rt, owner) = setup_scope("memo", at);
    let key = rt.create_memo(
        owner,
        Box::new(move |_| f()),
        Some(<T as PartialEq>::eq),
        at,
    );
    Value::node(rt.id(), key)
}

/// A user effect (Solid `createEffect`): runs after the turn's ops are
/// flushed and again whenever what it read changes. Register cleanups with
/// [`on_cleanup`] inside it. Setup only.
#[track_caller]
pub fn effect(f: impl Fn() + 'static) {
    let at = Location::caller();
    let (rt, owner) = setup_scope("effect", at);
    rt.create_effect(owner, EffectKind::User, Box::new(f), at);
}

/// Run `f` with the new value each time `source` changes, but not for the
/// initial value (Solid `on(source, f, { defer: true })`). If computing the
/// initial value fails, the error is reported and the next change is still
/// delivered. Only `source` is tracked: `f` runs untracked, so what it
/// reads is not a dependency. It is a user effect (after the turn's ops
/// are flushed). Setup only.
///
/// ```ignore
/// effect_on(board, move |snapshot| saver.push(snapshot.clone()));
/// ```
#[track_caller]
pub fn effect_on<T: Clone + 'static>(source: impl IntoValue<T>, f: impl Fn(&T) + 'static) {
    let at = Location::caller();
    let (rt, owner) = setup_scope("effect_on", at);
    let source = source.into_prop();
    rt.watch(owner, EffectKind::User, move || source.get(), f, at);
}

/// Register a cleanup (Solid `onCleanup`): during setup it runs when the
/// current owner (branch, row, root, ...) is disposed; inside an effect it
/// runs before the effect re-runs and when it is disposed.
#[track_caller]
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    let at = Location::caller();
    if let Some(rt) = current_runtime()
        && let Some(e) = rt.current_effect()
    {
        rt.add_effect_cleanup(e, Box::new(f));
        return;
    }
    let (rt, owner) = setup_scope("on_cleanup", at);
    rt.add_cleanup(owner, Box::new(f));
}

/// Run `f` once after the host has applied the nodes created by the
/// current setup (Solid `onMount`). `f` runs untracked. Setup only.
#[track_caller]
pub fn on_mount(f: impl FnOnce() + 'static) {
    let at = Location::caller();
    let (rt, owner) = setup_scope("on_mount", at);
    rt.push_mount(owner, Box::new(f));
}

/// A reactive value of the nearest `T` provided above the current owner (Solid
/// `useContext`); a Copy handle struct is copied, a larger value is shared
/// by wrapping it in an `Rc` before providing it.
/// Works in setup and in handlers, tasks and effects (the nearest owner
/// of the frame stack). Panics at the caller, naming `T`, if none is provided.
/// Use [`try_use_context`] when the context is optional.
#[track_caller]
pub fn use_context<T: Clone + 'static>() -> Value<T> {
    let at = Location::caller();
    let (rt, owner) = owner_scope("use_context", at);
    lookup_context::<Value<T>>(&rt, owner).unwrap_or_else(|| {
        misuse_at(
            ErrorKind::Message,
            format!(
                "`use_context::<{}>()` requires context; wrap this component in `provide(value, || view)`",
                type_name::<T>()
            ),
            at,
        )
    })
}

/// The nearest `T` provided above the current owner, or `None` if absent.
/// Works in setup and in handlers, tasks and effects, like [`use_context`].
/// A missing or disposed owner is still a located misuse.
#[track_caller]
pub fn try_use_context<T: Clone + 'static>() -> Option<Value<T>> {
    let at = Location::caller();
    let (rt, owner) = owner_scope("try_use_context", at);
    lookup_context::<Value<T>>(&rt, owner)
}

/// The nearest `T` provided to `owner` or one of its ancestors.
pub(crate) fn lookup_context<T: Clone + 'static>(rt: &Runtime, owner: Key) -> Option<T> {
    rt.find_context(owner, TypeId::of::<T>())
        .and_then(|v| v.downcast_ref::<T>().cloned())
}

/// A value-free dependency owned by the current component setup.
#[track_caller]
pub fn notifier() -> crate::Notifier {
    let at = Location::caller();
    let (rt, owner) = setup_scope("notifier", at);
    crate::Notifier::from_key(rt.id(), rt.create_source(owner, (), at))
}
