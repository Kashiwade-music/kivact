//! Role: Public Copy handles: Signal, Value, Var, Callback, Binding.
//! Every handle is `(AppId, key)` and is checked on use. `Selector` is in
//! `selector`; the setup functions (`signal`, `memo`, ...) are in `setup`.

use crate::reactive::AppId;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::panic::Location;
use std::rc::Rc;

use super::graph::{is_memo, source_cell};
use crate::reactive::arena::Key;
use crate::reactive::data::{Binding, Callback, Signal, Value, Var};
use crate::reactive::data::{BindingRepr, CallbackFn, Runtime, ValueRepr};
use crate::reactive::error::{ErrorKind, UiError, misuse, misuse_at, new_error};
use crate::reactive::registry::{runtime, try_runtime};

// ---------------------------------------------------------------------------
// Signal

impl<T: 'static> Signal<T> {
    pub(crate) fn from_key(app: AppId, key: Key) -> Self {
        Signal {
            app,
            key,
            _m: PhantomData,
        }
    }

    fn with_impl<R>(
        &self,
        track: bool,
        at: &'static Location<'static>,
        f: impl FnOnce(&T) -> R,
    ) -> R {
        let rt = runtime(self.app, at);
        let cell = rt.read_or_panic(self.key, track, "Signal", at);
        drop(rt);
        let Some(src) = source_cell::<T>(&cell) else {
            misuse_at(ErrorKind::WrongApp, "handle type mismatch", at)
        };
        let guard = match src.value.try_borrow() {
            Ok(g) => g,
            Err(_) => misuse_at(
                ErrorKind::BorrowedWrite,
                "signal read while it is being updated",
                at,
            ),
        };
        f(&guard)
    }

    /// Read (tracked) and clone.
    #[track_caller]
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with_impl(true, Location::caller(), T::clone)
    }

    /// Read and clone without subscribing.
    #[track_caller]
    pub fn get_untracked(&self) -> T
    where
        T: Clone,
    {
        self.with_impl(false, Location::caller(), T::clone)
    }

    /// Borrow the value (tracked).
    #[track_caller]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.with_impl(true, Location::caller(), f)
    }

    /// Borrow the value without subscribing.
    #[track_caller]
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.with_impl(false, Location::caller(), f)
    }

    /// `None` if the signal was disposed (never panics for that reason).
    #[track_caller]
    pub fn try_get(&self) -> Option<T>
    where
        T: Clone,
    {
        let at = Location::caller();
        if !crate::reactive::execution::handle_usable_here(self.app) {
            return None;
        }
        let rt = try_runtime(self.app)?;
        let cell = rt.read_node(self.key, true, at)?;
        let src = source_cell::<T>(&cell)?;
        let v = src.value.try_borrow().ok()?.clone();
        Some(v)
    }

    /// Read without subscribing and clone; `None` when the signal or its App
    /// is gone, or while the value is being updated. Never panics.
    pub fn try_get_untracked(&self) -> Option<T>
    where
        T: Clone,
    {
        let rt = try_runtime(self.app)?;
        let cell = rt.read_node(self.key, false, Location::caller())?;
        let src = source_cell::<T>(&cell)?;
        let v = src.value.try_borrow().ok()?.clone();
        Some(v)
    }

    fn write(&self, at: &'static Location<'static>, f: impl FnOnce(&mut T) -> bool) {
        let rt = runtime(self.app, at);
        rt.write_node(self.key, at, |cell| {
            let Some(src) = source_cell::<T>(cell) else {
                return false;
            };
            let mut guard = match src.value.try_borrow_mut() {
                Ok(g) => g,
                Err(_) => misuse_at(
                    ErrorKind::BorrowedWrite,
                    "signal written while borrowed by `with`",
                    at,
                ),
            };
            f(&mut guard)
        });
    }

    /// Set a new value; readers are notified only if it differs.
    #[track_caller]
    pub fn set(&self, value: T)
    where
        T: PartialEq,
    {
        self.write(Location::caller(), move |v| {
            if *v == value {
                false
            } else {
                *v = value;
                true
            }
        });
    }

    /// Mutate in place and return what `f` returns; always notifies. A
    /// disposed signal has no value for `f`, so updating one is a located
    /// `WriteAfterDispose` misuse (`set` and `notify` ignore it instead).
    #[track_caller]
    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let at = Location::caller();
        let mut out = None;
        self.write(at, |v| {
            out = Some(f(v));
            true
        });
        out.unwrap_or_else(|| {
            misuse_at(
                ErrorKind::WriteAfterDispose,
                "`Signal::update` on a disposed signal",
                at,
            )
        })
    }

    /// Notify readers without changing the value (external objects).
    #[track_caller]
    pub fn notify(&self) {
        self.write(Location::caller(), |_| true);
    }

    /// A two-way binding to one field of the model (setup only).
    /// `get_ref`/`get_mut` project the field; no clone closures are needed.
    /// Reading is a memo of the field, so readers are notified only when the
    /// field changes; writing replaces the field in place and notifies the
    /// signal only when the new value differs.
    #[track_caller]
    pub fn project<U: crate::PropValue>(
        self,
        get_ref: impl Fn(&T) -> &U + 'static,
        get_mut: impl Fn(&mut T) -> &mut U + 'static,
    ) -> Binding<U> {
        let at = Location::caller();
        let rt = runtime(self.app, at);
        let owner = rt.require_setup("Signal::project", at);
        let this = self;
        let memo = rt.create_memo(
            owner,
            Box::new(move |_| this.with(|t| get_ref(t).clone())),
            Some(<U as PartialEq>::eq),
            at,
        );
        let cb: Rc<CallbackFn<U>> = Rc::new(CallbackFn {
            owner,
            f: Box::new(move |u: U| {
                let mut changed = false;
                this.write(at, |t| {
                    let slot = get_mut(t);
                    if *slot != u {
                        *slot = u;
                        changed = true;
                    }
                    changed
                });
                Ok(())
            }),
        });
        let key = rt.insert_object(owner, cb, None, at);
        Binding {
            repr: BindingRepr::Pair(
                Value::node(self.app, memo),
                Callback {
                    app: self.app,
                    key,
                    _m: PhantomData,
                },
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Value

impl<T: 'static> Value<T> {
    pub(crate) fn node(app: AppId, key: Key) -> Self {
        Value {
            app,
            repr: ValueRepr::Node(key),
            _m: PhantomData,
        }
    }

    pub(crate) fn constant(app: AppId, key: Key) -> Self {
        Value {
            app,
            repr: ValueRepr::Const(key),
            _m: PhantomData,
        }
    }

    /// The value if this is a constant of a live App usable here
    /// (`into_value` of a constant), else `None`.
    pub(crate) fn constant_value(&self) -> Option<T>
    where
        T: Clone,
    {
        let ValueRepr::Const(key) = self.repr else {
            return None;
        };
        if !crate::reactive::execution::handle_usable_here(self.app) {
            return None;
        }
        try_runtime(self.app)?
            .const_value::<T>(key)
            .map(|v| T::clone(&v))
    }

    fn with_impl<R>(
        &self,
        track: bool,
        at: &'static Location<'static>,
        f: impl FnOnce(&T) -> R,
    ) -> R {
        let rt = runtime(self.app, at);
        let key = match self.repr {
            ValueRepr::Const(key) => {
                let Some(value) = rt.const_value::<T>(key) else {
                    rt.const_disposed_panic(key, at)
                };
                drop(rt);
                return f(&value);
            }
            ValueRepr::Node(key) => key,
        };
        let cell = rt.read_or_panic(key, track, "Value", at);
        if let Some(src) = source_cell::<T>(&cell) {
            drop(rt);
            let guard = match src.value.try_borrow() {
                Ok(g) => g,
                Err(_) => misuse_at(
                    ErrorKind::BorrowedWrite,
                    "value read while it is being updated",
                    at,
                ),
            };
            f(&guard)
        } else if is_memo::<T>(&cell) {
            rt.memo_value::<T, R>(&cell, f, at)
        } else {
            misuse_at(ErrorKind::WrongApp, "handle type mismatch", at)
        }
    }

    /// Read (tracked) and clone.
    #[track_caller]
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with_impl(true, Location::caller(), T::clone)
    }

    /// Read and clone without subscribing.
    #[track_caller]
    pub fn get_untracked(&self) -> T
    where
        T: Clone,
    {
        self.with_impl(false, Location::caller(), T::clone)
    }

    /// Borrow the value (tracked).
    #[track_caller]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.with_impl(true, Location::caller(), f)
    }

    /// Borrow the value without subscribing.
    #[track_caller]
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.with_impl(false, Location::caller(), f)
    }

    /// `None` if disposed.
    #[track_caller]
    pub fn try_get(&self) -> Option<T>
    where
        T: Clone,
    {
        if !crate::reactive::execution::handle_usable_here(self.app) {
            return None;
        }
        let rt = try_runtime(self.app)?;
        let alive = match self.repr {
            ValueRepr::Const(key) => rt.consts.borrow().contains(key),
            ValueRepr::Node(key) => rt.node_alive(key),
        };
        drop(rt);
        alive.then(|| self.get())
    }
}

// ---------------------------------------------------------------------------
// Var

impl<T: 'static> Var<T> {
    pub(super) fn from_key(app: AppId, key: Key) -> Self {
        Var {
            app,
            key,
            _m: PhantomData,
        }
    }

    fn cell(&self, at: &'static Location<'static>) -> Rc<RefCell<T>> {
        runtime(self.app, at).object_or_panic::<RefCell<T>>(self.key, "Var", at)
    }

    /// Read and clone (never tracked).
    #[track_caller]
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with(T::clone)
    }

    /// Borrow the value (never tracked).
    #[track_caller]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let at = Location::caller();
        let c = self.cell(at);
        let g = match c.try_borrow() {
            Ok(g) => g,
            Err(_) => misuse_at(ErrorKind::BorrowedWrite, "var read while being updated", at),
        };
        f(&g)
    }

    /// Replace the value. Never notifies.
    #[track_caller]
    pub fn set(&self, value: T) {
        self.update(move |v| *v = value)
    }

    /// Mutate in place and return what `f` returns. Never notifies. A write
    /// while the value is borrowed (`with`, `update`) is a located
    /// `BorrowedWrite` misuse.
    #[track_caller]
    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let at = Location::caller();
        let c = self.cell(at);
        let mut g = match c.try_borrow_mut() {
            Ok(g) => g,
            Err(_) => misuse_at(ErrorKind::BorrowedWrite, "var written while borrowed", at),
        };
        f(&mut g)
    }
}

// ---------------------------------------------------------------------------
// Callback

impl<E: 'static> Callback<E> {
    pub(super) fn from_key(app: AppId, key: Key) -> Self {
        Callback {
            app,
            key,
            _m: PhantomData,
        }
    }

    /// The callback, or the `WrongApp`/`Disposed` error of a stale handle.
    fn lookup(
        &self,
        at: &'static Location<'static>,
    ) -> Result<(Rc<Runtime>, Rc<CallbackFn<E>>), UiError> {
        let rt = match try_runtime(self.app) {
            Some(rt) if crate::reactive::execution::handle_usable_here(self.app) => rt,
            _ => {
                return Err(new_error(
                    ErrorKind::WrongApp,
                    "callback of another App (or of an App that no longer exists)",
                    Some(at),
                ));
            }
        };
        match rt.object::<CallbackFn<E>>(self.key) {
            Some(cb) => Ok((rt, cb)),
            None => {
                let tomb = rt.objects.borrow().tomb(self.key);
                Err(new_error(
                    ErrorKind::Disposed {
                        created_at: tomb.map(|t| t.created_at),
                        disposed_by: tomb.and_then(|t| t.disposed_by),
                    },
                    "Callback used after its owner was disposed",
                    Some(at),
                ))
            }
        }
    }

    fn invoke(rt: &Runtime, cb: &CallbackFn<E>, e: E) -> Result<(), UiError> {
        // Called from outside every App (tests, main): the call itself is the
        // boundary. Inside an App the callback runs for its own owner
        // wherever it is called from (`spawn`, `context` and the
        // `kivact::platform` functions use the callback's owner) and always
        // untracked: what it reads is not a dependency of a calling effect
        // or memo. Purity and the setup owner stay the caller's.
        let _g = if crate::reactive::execution::executing_app() == AppId::NONE {
            rt.handler_frame(Some(cb.owner))
        } else {
            rt.callback_frame(cb.owner)
        };
        (cb.f)(e)
    }

    /// Call; an `Err` goes to the nearest `on_error`.
    #[track_caller]
    pub fn call(&self, e: E) {
        let at = Location::caller();
        let (rt, cb) = match self.lookup(at) {
            Ok(x) => x,
            Err(err) => misuse(err),
        };
        if let Err(err) = Self::invoke(&rt, &cb, e) {
            rt.route(Some(cb.owner), err);
        }
    }

    /// Call and return the result; a disposed callback or one of another
    /// App is an `Err`, not a panic.
    #[track_caller]
    pub fn try_call(&self, e: E) -> Result<(), UiError> {
        let at = Location::caller();
        let (rt, cb) = self.lookup(at)?;
        Self::invoke(&rt, &cb, e)
    }

    /// A `Callback<()>` that calls this one with `e` (setup only).
    #[track_caller]
    pub fn bind(&self, e: E) -> Callback<()>
    where
        E: Clone,
    {
        let at = Location::caller();
        let rt = runtime(self.app, at);
        let owner = rt.require_setup("Callback::bind", at);
        let this = *self;
        let cb: Rc<CallbackFn<()>> = Rc::new(CallbackFn {
            owner,
            f: Box::new(move |()| this.try_call(e.clone())),
        });
        Callback {
            app: self.app,
            key: rt.insert_object(owner, cb, None, at),
            _m: PhantomData,
        }
    }

    /// A Copy closure calling this callback (for `on_*` handlers).
    pub fn to_fn(self) -> impl Fn(E) + Copy + 'static {
        move |e| self.call(e)
    }
}

// ---------------------------------------------------------------------------
// Binding

impl<T: crate::PropValue> Binding<T> {
    /// Read (tracked) and clone.
    #[track_caller]
    pub fn get(&self) -> T {
        self.with(T::clone)
    }

    /// Read and clone without tracking a dependency.
    #[track_caller]
    pub fn get_untracked(&self) -> T {
        self.with_untracked(T::clone)
    }

    /// Borrow the value (tracked).
    #[track_caller]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        match self.repr {
            BindingRepr::Signal(s) => s.with(f),
            BindingRepr::Pair(v, _) => v.with(f),
        }
    }

    /// Borrow the value without tracking a dependency.
    #[track_caller]
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        match self.repr {
            BindingRepr::Signal(s) => s.with_untracked(f),
            BindingRepr::Pair(v, _) => v.with_untracked(f),
        }
    }

    /// Write `v` through the binding (the signal, or the setter closure).
    #[track_caller]
    pub fn set(&self, v: T) {
        match self.repr {
            BindingRepr::Signal(s) => s.set(v),
            BindingRepr::Pair(_, cb) => cb.call(v),
        }
    }

    /// A two-way binding to one field of this binding's value (setup
    /// only), like [`Signal::project`]: reading projects the field, writing
    /// it writes the whole value back with the field replaced (nothing is
    /// written when the field is unchanged).
    #[track_caller]
    pub fn project<U: crate::PropValue>(
        self,
        get_ref: impl Fn(&T) -> &U + 'static,
        get_mut: impl Fn(&mut T) -> &mut U + 'static,
    ) -> Binding<U> {
        let (value, cb) = match self.repr {
            BindingRepr::Signal(s) => return s.project(get_ref, get_mut),
            BindingRepr::Pair(value, cb) => (value, cb),
        };
        let at = Location::caller();
        let rt = runtime(value.app, at);
        let owner = rt.require_setup("Binding::project", at);
        let memo = rt.create_memo(
            owner,
            Box::new(move |_| value.with(|t| get_ref(t).clone())),
            Some(<U as PartialEq>::eq),
            at,
        );
        let write: Rc<CallbackFn<U>> = Rc::new(CallbackFn {
            owner,
            f: Box::new(move |u: U| {
                let mut whole = value.get_untracked();
                let slot = get_mut(&mut whole);
                if *slot == u {
                    return Ok(());
                }
                *slot = u;
                cb.try_call(whole)
            }),
        });
        let key = rt.insert_object(owner, write, None, at);
        Binding {
            repr: BindingRepr::Pair(
                Value::node(value.app, memo),
                Callback::from_key(value.app, key),
            ),
        }
    }
}

impl<T: crate::PropValue> Binding<T> {
    /// This binding as a binding of `Option<T>`, for an input that edits an
    /// optional choice (`select`, `radio_group`). Reading yields
    /// `Some(current)`; writing `Some(v)` writes `v`; writing `None` is
    /// ignored (setup only).
    ///
    /// ```ignore
    /// select("Method", items, form.project(|f| &f.method, |f| &mut f.method).optional())
    /// ```
    #[track_caller]
    pub fn optional(self) -> Binding<Option<T>> {
        crate::reactive::setup::binding(
            move || Some(self.get()),
            move |v: Option<T>| {
                if let Some(v) = v {
                    self.set(v);
                }
            },
        )
    }
}

impl<T: crate::PropValue> Signal<T> {
    /// This signal as a binding of `Option<T>`, like [`Binding::optional`]:
    /// reading yields `Some(current)`, writing `Some(v)` writes `v`, writing
    /// `None` is ignored (setup only).
    #[track_caller]
    pub fn optional(self) -> Binding<Option<T>> {
        self.into_binding().optional()
    }
}

/// The read-only view of a signal: the same node, no new one.
impl<T: 'static> From<Signal<T>> for Value<T> {
    fn from(signal: Signal<T>) -> Self {
        Value::node(signal.app, signal.key)
    }
}

/// Bind the same value in both directions. For an f64 control, write `signal(40.0)`.
#[diagnostic::on_unimplemented(
    message = "this value cannot be bound as `{T}`",
    note = "pass a Signal or Binding of the edited type; an f64 control needs `signal(40.0)`"
)]
pub trait IntoBinding<T: 'static>: super::sealed::IntoBindingSeal {
    /// Convert this writable value into a binding.
    fn into_binding(self) -> Binding<T>;
}
impl<T: 'static> super::sealed::IntoBindingSeal for Signal<T> {}
impl<T: 'static> super::sealed::IntoBindingSeal for Binding<T> {}
impl<T: 'static> IntoBinding<T> for Signal<T> {
    fn into_binding(self) -> Binding<T> {
        Binding {
            repr: BindingRepr::Signal(self),
        }
    }
}
impl<T: 'static> IntoBinding<T> for Binding<T> {
    fn into_binding(self) -> Binding<T> {
        self
    }
}

impl<F, R> From<F> for Callback<()>
where
    F: Fn() -> R + 'static,
    R: crate::HandlerResult,
{
    #[track_caller]
    fn from(f: F) -> Self {
        let at = Location::caller();
        let (rt, owner) = super::scope::owner_scope("Callback::from", at);
        let cb: Rc<CallbackFn<()>> = Rc::new(CallbackFn {
            owner,
            f: Box::new(move |()| f().into_outcome().map(|_| ())),
        });
        Self::from_key(rt.id(), rt.insert_object(owner, cb, None, at))
    }
}

/// Project a nested writable field: `bind!(model.address.street)`.
#[macro_export]
macro_rules! bind {
    ($source:ident.$($field:ident).+) => {
        $source.project(|value| &value.$($field).+, |value| &mut value.$($field).+)
    };
}
