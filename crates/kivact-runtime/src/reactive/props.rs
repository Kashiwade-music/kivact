//! Role: Property value abstraction and render effects.
//!
//! `IntoValue`: every property accepts a constant, a `Signal`, a `Value`, a
//! `Binding`, an `Animated` or a closure, and a component reads it with
//! `into_value()`. Static values are passed to the engine's create call;
//! dynamic ones get one render effect each. A string prop is normalized to
//! an owned `String` (`&str` is a constant); a closure is `'static`.
//!
//! `IntoText`: the input of every display or accessible string. It also
//! takes a closure returning anything that becomes a `String`, and a
//! component reads it with `into_text_value()`.
//!
//! Invariant: a `Value` in the constant arena (`IntoValue::into_value`
//! of a constant) counts as static: its value never changes, so
//! [`Prop::fixed`] reads it once and no effect is created.

use kivact_types::Asset;
use std::panic::Location;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use crate::reactive::Owner;
use crate::reactive::data::{Binding, BindingRepr, Signal, Value};

enum PropRepr<T: 'static> {
    Static(T),
    Value(Value<T>),
    Fn(Rc<dyn Fn() -> T>, fn(&T, &T) -> bool),
}

/// A property input as a component received it: a constant, an existing
/// [`Value`], or a closure computed on every read ([`IntoValue::into_prop`]).
/// It describes the input only; no graph node exists until
/// [`Prop::into_value`] makes one.
pub struct Prop<T: 'static>(PropRepr<T>);

impl<T: Clone + 'static> Clone for Prop<T> {
    fn clone(&self) -> Self {
        Prop(match &self.0 {
            PropRepr::Static(v) => PropRepr::Static(v.clone()),
            PropRepr::Value(v) => PropRepr::Value(*v),
            PropRepr::Fn(f, eq) => PropRepr::Fn(f.clone(), *eq),
        })
    }
}

impl<T: 'static> Prop<T> {
    /// A fixed value. No graph node is created.
    pub fn constant(value: T) -> Self {
        Self(PropRepr::Static(value))
    }

    /// A reactive value read as it is.
    pub fn value(value: Value<T>) -> Self {
        Self(PropRepr::Value(value))
    }

    /// A value `f` computes on every read; equal results (`PartialEq`) do
    /// not notify once it is made a memo.
    // xtask-allow(internal-contract): Core mounting adapts deferred properties across the runtime boundary.
    #[doc(hidden)]
    pub fn computed(f: Rc<dyn Fn() -> T>) -> Self
    where
        T: PartialEq,
    {
        Self(PropRepr::Fn(f, T::eq))
    }

    /// The value if it can never change: a constant, or a `Value` that no
    /// handle can write ([`crate::reactive::Owner::constant`]). Such a property is
    /// written once, without an effect. The read is not tracked.
    pub fn fixed(&self) -> Option<T>
    where
        T: Clone,
    {
        match &self.0 {
            PropRepr::Static(v) => Some(v.clone()),
            PropRepr::Value(v) => v.constant_value(),
            PropRepr::Fn(_, _) => None,
        }
    }

    /// The current value (tracked when called inside a computation).
    #[track_caller]
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        match &self.0 {
            PropRepr::Static(v) => v.clone(),
            PropRepr::Value(v) => v.get(),
            PropRepr::Fn(f, _) => f(),
        }
    }

    /// Run `f` on the current value without cloning it (tracked).
    #[track_caller]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        match &self.0 {
            PropRepr::Static(v) => f(v),
            PropRepr::Value(v) => v.with(f),
            PropRepr::Fn(g, _) => f(&g()),
        }
    }

    /// A shareable closure reading the value (for sources several effects
    /// read). A value that can never change is read once here, so the
    /// closure tracks nothing.
    // xtask-allow(internal-contract): Core mounting adapts deferred properties across the runtime boundary.
    #[doc(hidden)]
    pub fn into_fn(self) -> Rc<dyn Fn() -> T>
    where
        T: Clone,
    {
        match self.0 {
            PropRepr::Static(v) => Rc::new(move || v.clone()),
            PropRepr::Value(v) => match v.constant_value() {
                Some(c) => Rc::new(move || c.clone()),
                None => Rc::new(move || v.get()),
            },
            PropRepr::Fn(f, _) => f,
        }
    }

    /// This input as a [`Value`]: a reactive value as it is, a constant as a
    /// new unwritable value of `owner`, a closure as a new memo of `owner`.
    /// `at` is where diagnostics locate a new node.
    // xtask-allow(internal-contract): Core mounting adapts deferred properties across the runtime boundary.
    #[doc(hidden)]
    pub fn into_value(self, owner: Owner, at: &'static Location<'static>) -> Value<T> {
        match self.0 {
            PropRepr::Value(v) => v,
            PropRepr::Static(v) => owner.constant(v, at),
            PropRepr::Fn(f, eq) => {
                let rt = crate::reactive::registry::runtime(owner.app, at);
                Value::node(
                    owner.app,
                    rt.create_memo(owner.key, Box::new(move |_| f()), Some(eq), at),
                )
            }
        }
    }
}

/// Anything a property accepts: constants of the concrete property type,
/// `Signal<T>`, `Value<T>`, `Binding<T>`, `Animated<T>` and `Fn() -> T`
/// closures. A closure result must implement `PartialEq` so its memo
/// suppresses notifications for equal results. A component's own value
/// type accepts constants once it derives `Clone` and `PartialEq` and is
/// listed in [`constant_value!`](crate::reactive::constant_value) (the component
/// author's guide).
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as a `{T}` property",
    note = "pass a `{T}`, a closure `move || ..` returning one, or a `Signal`/`Value`/`Binding` of it",
    note = "a closure returns an owned `{T}`, not a borrow of one (`.to_owned()`, `format!(..)`)"
)]
pub trait IntoValue<T: 'static> {
    /// This input as a [`Prop`], without creating a graph node.
    // xtask-allow(internal-contract): Cross-crate IntoValue implementations and constant_value! expansion describe deferred inputs.
    #[doc(hidden)]
    fn into_prop(self) -> Prop<T>;

    /// This input as a [`Value`], to read it in a component (twice, or in
    /// a closure) without knowing what the caller passed. A `Value`,
    /// `Signal` or `Animated` stays the same node; a closure becomes a memo
    /// of the current owner; a constant becomes a writer-less value, which
    /// a prop binds as cheaply as the constant itself (no effect).
    ///
    /// ```ignore
    /// #[track_caller]
    /// pub fn badge(label: impl IntoValue<String>) -> Node {
    ///     let label = label.into_value();
    ///     node().name(label).child(text(label))
    /// }
    /// ```
    ///
    /// Existing reactive handles are returned unchanged in any phase.
    /// Constants and closures need setup (a closure makes a memo); elsewhere
    /// they raise a `SetupOnly` error located at the caller.
    #[track_caller]
    fn into_value(self) -> Value<T>
    where
        Self: Sized,
    {
        match self.into_prop().0 {
            PropRepr::Value(value) => value,
            source => {
                let at = Location::caller();
                let owner = Owner::current("IntoValue::into_value", at);
                Prop(source).into_value(owner, at)
            }
        }
    }
}

/// A text input: a constant (`&str`, `String`, `Rc<str>`), a reactive source of a
/// `String` (`Signal`, `Value`, `Binding`, a stored `Reactive`), or a closure returning anything that
/// becomes a `String` (`move || if on.get() { "on" } else { "off" }`).
///
/// Every public parameter that shows text or names something for assistive
/// technology (`text`, `.name(..)`, a component's label, placeholder or hint)
/// takes it. A closure result is converted with `Into<String>` on each
/// evaluation, and equal strings do not notify. Text that is data, such as an
/// editable value bound to a model, still takes [`IntoValue<String>`].
///
/// A component that takes `impl IntoText` reads it once with
/// [`into_text_value`](IntoText::into_text_value), the counterpart of
/// [`IntoValue::into_value`].
///
/// This trait is sealed: the listed types are all there are.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as text",
    note = "pass a `&str`, a `String`, a `Signal`/`Value`/`Binding` of a `String`, or a closure `move || ..` returning a `&str` or `String`",
    note = "format a number first: `move || count.get().to_string()`"
)]
pub trait IntoText: crate::reactive::sealed::IntoTextSeal {
    /// This input as a [`Prop`], without creating a graph node.
    // xtask-allow(internal-contract): Cross-crate IntoText implementations describe deferred text inputs.
    #[doc(hidden)]
    fn into_text(self) -> Prop<String>;

    /// This input as a [`Value`], to read it in a component without knowing
    /// what the caller passed. It behaves as [`IntoValue::into_value`] does:
    /// an existing handle stays the same node, a constant becomes a
    /// writer-less value, a closure becomes a memo of the current owner (both
    /// need setup, and raise a `SetupOnly` error located at the caller
    /// elsewhere).
    ///
    /// ```ignore
    /// #[track_caller]
    /// pub fn badge(label: impl IntoText) -> Node {
    ///     let label = label.into_text_value();
    ///     node().name(label).child(text(label))
    /// }
    /// ```
    #[track_caller]
    fn into_text_value(self) -> Value<String>
    where
        Self: Sized,
    {
        match self.into_text().0 {
            PropRepr::Value(value) => value,
            source => {
                let at = Location::caller();
                let owner = Owner::current("IntoText::into_text_value", at);
                Prop(source).into_value(owner, at)
            }
        }
    }
}

impl crate::reactive::sealed::IntoTextSeal for &str {}
impl crate::reactive::sealed::IntoTextSeal for String {}
impl crate::reactive::sealed::IntoTextSeal for Rc<str> {}
impl crate::reactive::sealed::IntoTextSeal for Signal<String> {}
impl crate::reactive::sealed::IntoTextSeal for Value<String> {}
impl crate::reactive::sealed::IntoTextSeal for Binding<String> {}
impl crate::reactive::sealed::IntoTextSeal for Reactive<String> {}
impl<R: Into<String>, F: Fn() -> R + 'static> crate::reactive::sealed::IntoTextSeal for F {}

impl IntoText for &str {
    fn into_text(self) -> Prop<String> {
        Prop(PropRepr::Static(self.to_owned()))
    }
}

impl IntoText for String {
    fn into_text(self) -> Prop<String> {
        Prop(PropRepr::Static(self))
    }
}

impl IntoText for Rc<str> {
    fn into_text(self) -> Prop<String> {
        Prop(PropRepr::Static(self.as_ref().to_owned()))
    }
}

impl IntoText for Signal<String> {
    fn into_text(self) -> Prop<String> {
        self.into_prop()
    }
}

impl IntoText for Value<String> {
    fn into_text(self) -> Prop<String> {
        self.into_prop()
    }
}

impl IntoText for Binding<String> {
    fn into_text(self) -> Prop<String> {
        self.into_prop()
    }
}

/// A text kept as data, such as the label of a choice option in application
/// state.
impl IntoText for Reactive<String> {
    fn into_text(self) -> Prop<String> {
        self.into_prop()
    }
}

// Not recommended in diagnostics, as for `IntoValue`: a wrong input is
// reported once, as not text, instead of as a closure with the wrong return
// type.
#[diagnostic::do_not_recommend]
impl<R: Into<String>, F: Fn() -> R + 'static> IntoText for F {
    fn into_text(self) -> Prop<String> {
        Prop(PropRepr::Fn(Rc::new(move || self().into()), String::eq))
    }
}

/// A reactive input: a `Signal`, `Value`, `Binding`, `Animated` or a
/// `Fn() -> T` closure, excluding raw scalar constants. This marker uses
/// [`IntoValue::into_prop`] for ownerless conversion. Control flow (`show`, `show_some`,
/// `dynamic`) takes it: a branch that never changes needs no control flow,
/// and a constant there is usually a value read once by mistake
/// (`show(open.get(), ..)` instead of `show(open, ..)`).
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a reactive `{T}`",
    label = "a constant here would never change",
    note = "pass the reactive value itself (`open`, a `Signal`, `Value` or `Binding`), or a closure that reads it (`move || open.get()`)"
)]
pub trait IntoReactive<T: 'static>: IntoValue<T> {}

impl<T: 'static> IntoReactive<T> for Value<T> {}
impl<T: 'static> IntoReactive<T> for Signal<T> {}
impl<T: Clone + PartialEq + 'static> IntoReactive<T> for Binding<T> {}

// Keep raw scalar constants outside the reactive control-flow boundary.
#[diagnostic::do_not_recommend]
impl<T: PartialEq + 'static, F: Fn() -> T + 'static> IntoReactive<T> for F {}

/// A property input that can be stored and built anywhere: a constant, a
/// closure, or an existing `Value` or `Signal`. It has no owner, so creating
/// it needs no setup; normalize it to a [`Value`] with
/// [`IntoValue::into_value`].
///
/// Closure results implement `PartialEq` for memo equality on normalization.
/// Use it for plain data that carries a label which may change, such as a
/// choice option kept in application state. A [`Value`] is an owner-bound
/// handle to a graph node or constant arena entry; creating either needs
/// setup, while existing handles retain their identity in any phase.
/// A `Reactive` is ordinary
/// data that is cloned, compared by its holder and kept for as long as the
/// holder lives. The standard components take it for the labels of
/// `ListItem` and `MenuEntry`, and turn it into a `Value` of the using
/// component's owner with `into_value()` in its setup.
///
/// Use the public facade's `kivact::view::Reactive` in application code.
pub struct Reactive<T: 'static>(Prop<T>);

impl<T: Clone + 'static> Clone for Reactive<T> {
    fn clone(&self) -> Self {
        Reactive(self.0.clone())
    }
}

impl<T: 'static> Reactive<T> {
    /// Wrap a property input. Never requires setup.
    pub fn new(input: impl IntoValue<T>) -> Self {
        Reactive(input.into_prop())
    }

    /// Current value; tracked when called inside a computation.
    #[track_caller]
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.0.get()
    }

    /// Current value without tracking.
    #[track_caller]
    pub fn get_untracked(&self) -> T
    where
        T: Clone,
    {
        crate::reactive::frame::untrack(|| self.0.get())
    }
}

/// Equal constants are equal; a `Value` or `Signal` is equal to the same
/// node, and a closure to a clone of itself. Different kinds of input are
/// never equal, even when they currently read the same value.
impl<T: PartialEq + 'static> PartialEq for Reactive<T> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0.0, &other.0.0) {
            (PropRepr::Static(a), PropRepr::Static(b)) => a == b,
            (PropRepr::Value(a), PropRepr::Value(b)) => a == b,
            (PropRepr::Fn(a, _), PropRepr::Fn(b, _)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

/// An input already described as a [`Prop`] (`IntoText::into_text`) goes on
/// unchanged.
impl<T: 'static> IntoValue<T> for Prop<T> {
    fn into_prop(self) -> Prop<T> {
        self
    }
}

impl<T: Clone + 'static> IntoValue<T> for Reactive<T> {
    fn into_prop(self) -> Prop<T> {
        self.0
    }
}

impl<T: std::fmt::Debug + 'static> std::fmt::Debug for Reactive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0.0 {
            PropRepr::Static(v) => write!(f, "Reactive({v:?})"),
            _ => f.write_str("<reactive>"),
        }
    }
}

impl<T: 'static> IntoValue<T> for Value<T> {
    fn into_prop(self) -> Prop<T> {
        Prop(PropRepr::Value(self))
    }
}

impl<T: 'static> IntoValue<T> for Signal<T> {
    fn into_prop(self) -> Prop<T> {
        Prop(PropRepr::Value(Value::node(self.app, self.key)))
    }
}

impl<T: Clone + PartialEq + 'static> IntoValue<T> for Binding<T> {
    fn into_prop(self) -> Prop<T> {
        Prop(PropRepr::Value(match self.repr {
            BindingRepr::Signal(signal) => Value::node(signal.app, signal.key),
            BindingRepr::Pair(value, _) => value,
        }))
    }
}

// Not recommended in diagnostics: a wrong input is reported once, as not a
// property input, instead of as a closure with the wrong return type.
#[diagnostic::do_not_recommend]
impl<T: PartialEq + 'static, F: Fn() -> T + 'static> IntoValue<T> for F {
    fn into_prop(self) -> Prop<T> {
        Prop(PropRepr::Fn(Rc::new(self), T::eq))
    }
}

// Not recommended in diagnostics, as for `constant_value!`.
#[diagnostic::do_not_recommend]
impl IntoValue<String> for &str {
    fn into_prop(self) -> Prop<String> {
        Prop(PropRepr::Static(self.to_string()))
    }
}

/// A shared text: copied once, then cloned without copying.
impl IntoValue<Rc<str>> for &str {
    fn into_prop(self) -> Prop<Rc<str>> {
        Prop(PropRepr::Static(Rc::from(self)))
    }
}

impl IntoValue<Rc<str>> for String {
    fn into_prop(self) -> Prop<Rc<str>> {
        Prop(PropRepr::Static(Rc::from(self)))
    }
}

impl<T: Clone + 'static> IntoValue<Vec<T>> for Vec<T> {
    fn into_prop(self) -> Prop<Vec<T>> {
        Prop(PropRepr::Static(self))
    }
}

impl<T: Clone + 'static, const N: usize> IntoValue<Vec<T>> for [T; N] {
    fn into_prop(self) -> Prop<Vec<T>> {
        Prop(PropRepr::Static(self.into()))
    }
}

impl<T: Clone + 'static> IntoValue<Option<T>> for Option<T> {
    fn into_prop(self) -> Prop<Option<T>> {
        Prop(PropRepr::Static(self))
    }
}

/// Implements `IntoValue<T> for T` for each listed type, so a component can
/// accept its own `Clone + PartialEq` type as a prop, as a constant and as a
/// signal, `Value` or closure alike. Use it once per type, in the crate that
/// defines the type. Kivact's own crates and external component crates use
/// the same macro, through `kivact::state::constant_value!`.
#[macro_export]
macro_rules! constant_value {
    ($($t:ty),*) => {$(
        // Not recommended in diagnostics: a wrong input is reported once,
        // not as a list of every registered type.
        #[diagnostic::do_not_recommend]
        impl $crate::IntoValue<$t> for $t {
            fn into_prop(self) -> $crate::Prop<$t> {
                $crate::Prop::constant(self)
            }
        }
    )*};
}

// `()`: a content that never changes (a primitive measured by its
// constraints alone).
constant_value!(
    (),
    String,
    bool,
    f32,
    f64,
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    u64,
    usize,
    char
);

// A shared text: native editors take one and know it by identity.
constant_value!(Rc<str>);

constant_value!(
    kivact_types::TextDecorationStyle,
    kivact_types::Align,
    kivact_types::FlexDirection,
    kivact_types::Justify,
    kivact_types::Asset,
    kivact_types::Pixels,
    kivact_types::Point,
    kivact_types::Size,
    kivact_types::ImageFit,
    kivact_types::LiveRegion,
    kivact_types::SortDirection,
    kivact_types::Orientation,
    kivact_types::ScrollAxes,
    kivact_types::Placement,
    kivact_types::OverlayAlign,
    kivact_types::Rect,
    kivact_types::SemanticsText,
    std::ops::Range<usize>
);

/// A constant asset: `IntoValue<Asset>` for every type `Asset` converts
/// from (the conversion happens once, when the property is set).
macro_rules! asset_into_value {
    ($(impl[$($g:tt)*] $t:ty;)*) => {$(
        // Kept out of other properties' diagnostics: a byte slice or path is
        // only ever an asset.
        #[diagnostic::do_not_recommend]
        impl<$($g)*> IntoValue<Asset> for $t {
            fn into_prop(self) -> Prop<Asset> {
                Prop::constant(Asset::from(self))
            }
        }
    )*};
}

asset_into_value! {
    impl[const N: usize] &'static [u8; N];
    impl[] &'static [u8];
    impl[] Vec<u8>;
    impl[] Arc<[u8]>;
    impl[] &'static str;
    impl[] String;
    impl[] PathBuf;
    impl['a] &'a Path;
}

/// Derive `Clone`, `PartialEq`, `Eq` and `Hash` for collection keys.
#[diagnostic::on_unimplemented(
    message = "a collection key needs Clone + PartialEq + Eq + Hash",
    note = "add `#[derive(Clone, PartialEq, Eq, Hash)]` to your key type"
)]
pub trait ItemKey: Clone + Eq + std::hash::Hash + 'static {}
impl<T: Clone + Eq + std::hash::Hash + 'static> ItemKey for T {}

/// Derive `Clone` and `PartialEq` for values used as reactive properties.
#[diagnostic::on_unimplemented(
    message = "a property value needs Clone + PartialEq",
    note = "add `#[derive(Clone, PartialEq)]` to your value type"
)]
pub trait PropValue: Clone + PartialEq + 'static {}
impl<T: Clone + PartialEq + 'static> PropValue for T {}

impl IntoValue<f64> for i32 {
    fn into_prop(self) -> Prop<f64> {
        Prop::constant(f64::from(self))
    }
}
impl IntoValue<f64> for u32 {
    fn into_prop(self) -> Prop<f64> {
        Prop::constant(f64::from(self))
    }
}
impl IntoValue<f64> for f32 {
    fn into_prop(self) -> Prop<f64> {
        Prop::constant(f64::from(self))
    }
}
constant_value!(std::time::Duration, std::path::PathBuf);
