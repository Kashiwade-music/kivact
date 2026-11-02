//! Role: Control flow: show, show_some and dynamic.
//! A false/None/removed
//! branch disposes its owner; a continuing branch keeps its owner and only
//! its `Value` changes.
//!
//! The branch builders take no setup context: each branch is set
//! up in a `Setup` frame of its own fresh owner (`mount_branch`), so the
//! free setup functions and plain-function components work inside it.
//!
//! Invariants:
//! - Conditions, `show_some` values and `dynamic` sources are evaluated in
//!   a pure section (tracked, no side effects); building, mounting and
//!   disposing happen outside it.
//! - Branch builders run untracked in the branch's setup frame; a panic there
//!   propagates and ends the App.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_wire::NodeId;

use crate::view::boundary::mount_in;
use crate::view::descriptor::{IntoView, View, view_fn};
use crate::view::state::ViewState;
use kivact_reactive::EffectKind;
use kivact_reactive::{IntoReactive, IntoValue, Prop as InputProp};
use kivact_reactive::{Owner, Runtime};
use kivact_reactive::{Signal, Value};

/// A branch builder: runs in the setup frame of the branch's owner.
pub(crate) type Build = Rc<dyn Fn() -> View>;

pub(crate) struct Branch {
    pub(crate) owner: Owner,
    pub(crate) node: NodeId,
}

/// Mount `build`'s view in the fresh branch owner `owner` (a child owner the caller made, so it
/// can create the branch's state in it first), appended under `frag`.
pub(crate) fn mount_branch(
    rt: &Rc<Runtime>,
    owner: Owner,
    frag: NodeId,
    build: impl FnOnce() -> View,
) -> Branch {
    let node = mount_in(rt, owner, build);
    ViewState::tree_mut(rt).insert(frag, node, None);
    Branch { owner, node }
}

pub(crate) fn unmount_branch(rt: &Runtime, b: Branch) {
    // The records (their handlers) drop after the tree borrow, while the
    // owner still lives: a `Drop` there may use the tree and the owner.
    let removed = ViewState::tree_mut(rt).remove(b.node);
    drop(removed);
    b.owner.dispose();
}

/// A boxed branch builder from a closure returning any view.
pub(crate) fn build<V: IntoView>(f: impl Fn() -> V + 'static) -> Build {
    Rc::new(move || f().into_view())
}

// ---------------------------------------------------------------------------
// show

/// Conditional view. Built by [`show`].
pub struct Show {
    pub(crate) when: InputProp<bool>,
    pub(crate) child: Build,
    pub(crate) fallback: Option<Build>,
    pub(crate) at: &'static Location<'static>,
}

/// Render `child()` while `when` is true; the branch's owner (and the state
/// its components create) lives while `when` stays true. `when` is reactive
/// (a `Signal`, `Value`, `Binding` or closure): a constant is a compile
/// error, as a branch that never changes needs no `show`.
///
/// ```ignore
/// show(logged_in, dashboard).fallback(|| text("Please sign in"))
/// ```
#[track_caller]
pub fn show<V: IntoView>(when: impl IntoReactive<bool>, child: impl Fn() -> V + 'static) -> Show {
    Show {
        when: when.into_prop(),
        child: build(child),
        fallback: None,
        at: Location::caller(),
    }
}

impl Show {
    /// Render `f()` while `when` is false.
    pub fn fallback<V: IntoView>(mut self, f: impl Fn() -> V + 'static) -> Self {
        self.fallback = Some(build(f));
        self
    }
}

/// What a `show` has mounted.
enum ShowArm {
    /// Nothing: before the first run, and while the effect replaces the branch.
    Pending,
    /// The child, while the condition is true.
    Child(Branch),
    /// The fallback (if any), while the condition is false.
    Fallback(Option<Branch>),
}

impl ShowArm {
    fn into_branch(self) -> Option<Branch> {
        match self {
            ShowArm::Pending | ShowArm::Fallback(None) => None,
            ShowArm::Child(branch) | ShowArm::Fallback(Some(branch)) => Some(branch),
        }
    }
}

impl IntoView for Show {
    fn into_view(self) -> View {
        let Show {
            when,
            child,
            fallback,
            at,
        } = self;
        view_fn(move |cx| {
            let frag = cx.fragment();
            let rt = cx.weak();
            let parent = cx.owner;
            let state = RefCell::new(ShowArm::Pending);
            cx.effect(EffectKind::Internal, at, move || {
                let Some(rt) = rt.upgrade() else { return };
                // The condition only computes a value: pure.
                let when = kivact_reactive::pure(|| when.get());
                if matches!(
                    (when, &*state.borrow()),
                    (true, ShowArm::Child(_)) | (false, ShowArm::Fallback(_))
                ) {
                    return;
                }
                let old = std::mem::replace(&mut *state.borrow_mut(), ShowArm::Pending);
                if let Some(b) = old.into_branch() {
                    unmount_branch(&rt, b);
                }
                let next = if when {
                    ShowArm::Child(mount_branch(&rt, parent.child(at), frag, || child()))
                } else {
                    ShowArm::Fallback(
                        fallback
                            .as_ref()
                            .map(|f| mount_branch(&rt, parent.child(at), frag, || f())),
                    )
                };
                *state.borrow_mut() = next;
            });
            frag
        })
    }
}

// ---------------------------------------------------------------------------
// show_some

/// View for `Some` values. Built by [`show_some`].
pub struct ShowSome<T: 'static> {
    pub(crate) value: InputProp<Option<T>>,
    pub(crate) child: Rc<dyn Fn(Value<T>) -> View>,
    pub(crate) fallback: Option<Build>,
    pub(crate) at: &'static Location<'static>,
}

/// Render `child(value)` while the source is `Some`. Some→Some keeps the
/// branch (and its state) and only updates the `Value<T>`. The source is
/// reactive, as [`show`]'s condition.
///
/// ```ignore
/// fn task_editor(id: Value<u64>) -> impl IntoView { .. }
/// show_some(selected, task_editor).fallback(|| text("Nothing selected"))
/// ```
#[track_caller]
pub fn show_some<T: Clone + 'static, V: IntoView>(
    value: impl IntoReactive<Option<T>>,
    child: impl Fn(Value<T>) -> V + 'static,
) -> ShowSome<T> {
    ShowSome {
        value: value.into_prop(),
        child: Rc::new(move |v| child(v).into_view()),
        fallback: None,
        at: Location::caller(),
    }
}

impl<T: 'static> ShowSome<T> {
    /// Render `f()` while the source is `None`.
    pub fn fallback<V: IntoView>(mut self, f: impl Fn() -> V + 'static) -> Self {
        self.fallback = Some(build(f));
        self
    }
}

/// What a `show_some` has mounted.
enum SomeArm<T: 'static> {
    /// Nothing: before the first run, and while the effect replaces the branch.
    Pending,
    /// The child and the signal its `Value<T>` reads, while the source is `Some`.
    Child { branch: Branch, value: Signal<T> },
    /// The fallback (if any), while the source is `None`.
    Fallback(Option<Branch>),
}

impl<T: 'static> SomeArm<T> {
    fn into_branch(self) -> Option<Branch> {
        match self {
            SomeArm::Pending | SomeArm::Fallback(None) => None,
            SomeArm::Child { branch, .. } | SomeArm::Fallback(Some(branch)) => Some(branch),
        }
    }
}

impl<T: Clone + 'static> IntoView for ShowSome<T> {
    fn into_view(self) -> View {
        let ShowSome {
            value,
            child,
            fallback,
            at,
        } = self;
        view_fn(move |cx| {
            let frag = cx.fragment();
            let rt = cx.weak();
            let parent = cx.owner;
            let state: RefCell<SomeArm<T>> = RefCell::new(SomeArm::Pending);
            cx.effect(EffectKind::Internal, at, move || {
                let Some(rt) = rt.upgrade() else { return };
                let v = kivact_reactive::pure(|| value.get());
                let child_value = match &*state.borrow() {
                    SomeArm::Child { value, .. } => Some(*value),
                    _ => None,
                };
                let shows_fallback = matches!(*state.borrow(), SomeArm::Fallback(_));
                match (v, child_value) {
                    // Some->Some keeps the branch and only updates its value.
                    (Some(v), Some(sig)) => {
                        sig.update(move |slot| *slot = v);
                    }
                    (None, _) if shows_fallback => {}
                    (v, _) => {
                        let old = std::mem::replace(&mut *state.borrow_mut(), SomeArm::Pending);
                        if let Some(b) = old.into_branch() {
                            unmount_branch(&rt, b);
                        }
                        let next = match v {
                            Some(v) => {
                                let owner = parent.child(at);
                                let value = owner.signal(v, at);
                                let child = child.clone();
                                let branch = mount_branch(&rt, owner, frag, || {
                                    child(value.into_value())
                                });
                                SomeArm::Child { branch, value }
                            }
                            None => SomeArm::Fallback(
                                fallback
                                    .as_ref()
                                    .map(|f| mount_branch(&rt, parent.child(at), frag, || f())),
                            ),
                        };
                        *state.borrow_mut() = next;
                    }
                }
            });
            frag
        })
    }
}

// ---------------------------------------------------------------------------
// dynamic

/// Rebuild `child(value)` in a fresh branch each time the value of
/// `source` changes (Solid `<Dynamic>`). A notification with a value equal
/// (`PartialEq`) to the previous one keeps the branch and its state. Only
/// `source` is tracked (in a pure section); `child` runs untracked in the
/// branch's setup frame, so components called inside it may create state,
/// which lives until the next rebuild. `source` is reactive, as [`show`]'s
/// condition.
///
/// ```ignore
/// dynamic(mode, |mode| match mode {
///     Mode::List => list_view().into_view(),
///     Mode::Grid => grid_view().into_view(),
/// })
/// ```
#[track_caller]
pub fn dynamic<T: Clone + PartialEq + 'static, V: IntoView>(
    source: impl IntoReactive<T>,
    child: impl Fn(T) -> V + 'static,
) -> View {
    let at = Location::caller();
    let source = source.into_prop();
    view_fn(move |cx| {
        let frag = cx.fragment();
        let rt = cx.weak();
        let parent = cx.owner;
        // (value the current branch was built from, the branch)
        let current: RefCell<Option<(T, Branch)>> = RefCell::new(None);
        cx.effect(EffectKind::Internal, at, move || {
            let Some(rt) = rt.upgrade() else { return };
            let value = kivact_reactive::pure(|| source.get());
            if current.borrow().as_ref().is_some_and(|(v, _)| *v == value) {
                return;
            }
            if let Some((_, b)) = current.borrow_mut().take() {
                unmount_branch(&rt, b);
            }
            let built_from = value.clone();
            let b = mount_branch(&rt, parent.child(at), frag, || child(value).into_view());
            *current.borrow_mut() = Some((built_from, b));
        });
        frag
    })
}
