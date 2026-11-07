//! Role: Control flow: show, show_some and dynamic.
//! A false/None/removed
//! branch disposes its owner; a continuing branch keeps its owner and only
//! its `Value` changes.
//!
//! The branch builders take no setup context: each branch is set
//! up in a `Setup` frame of its own fresh owner (`mount_branch`), so the
//! free setup functions and plain-function components work inside it.
//!
//! Presence: a replaced branch leaves as its [`PresenceMode`] says. In
//! `Wait` mode the new branch mounts when the old one's exit completes
//! (`ExitComplete`), so the two contents do not overlap.
//!
//! Invariants:
//! - Conditions, `show_some` values and `dynamic` sources are evaluated in
//!   a pure section (tracked, no side effects); building, mounting and
//!   disposing happen outside it.
//! - Branch builders run untracked in the branch's setup frame; a panic there
//!   propagates and ends the App.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::{Rc, Weak};

use kivact_types::NodeId;

use crate::view::ExitLayout;
use crate::view::boundary::mount_in;
use crate::view::descriptor::{IntoView, View, view_fn};
use crate::view::state::ViewState;
use kivact_runtime::EffectKind;
use kivact_runtime::{Handler, IntoReactive, IntoValue, Prop as InputProp, PropValue};
use kivact_runtime::{Owner, Runtime};
use kivact_runtime::{Signal, Value};

/// A branch builder: runs in the setup frame of the branch's owner.
pub(crate) type Build = crate::view::ViewFn<()>;

pub(crate) struct Branch {
    pub(crate) owner: Owner,
    pub(crate) node: NodeId,
}

/// How the content of a branch leaves when the branch is replaced.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PresenceMode {
    /// The old content leaves while the new one enters.
    #[default]
    Sync,
    /// The new content mounts after the old content's exit completes.
    Wait,
    /// The old content leaves out of layout (`ExitLayout::Pop`) while the new one enters.
    PopLayout,
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
    ViewState::tree(rt).engine().insert(frag, node, None);
    Branch { owner, node }
}

pub(crate) fn unmount_branch(rt: &Runtime, b: Branch) {
    // The records (their handlers) drop after the tree borrow, while the
    // owner still lives: a `Drop` there may use the tree and the owner.
    let removed = ViewState::tree_mut(rt).remove(b.node);
    removed.commit(rt);
    b.owner.dispose();
}

/// A boxed branch builder from a closure returning any view.
pub(crate) fn build<V: IntoView>(f: impl crate::view::IntoViewFn<V>) -> Build {
    f.into_view_fn()
}

/// Where the branches of one flow mount: its fragment, the owner each
/// branch is a child of, the call site and the presence mode.
struct Site {
    rt: Weak<Runtime>,
    frag: NodeId,
    parent: Owner,
    at: &'static Location<'static>,
    presence: PresenceMode,
}

impl Site {
    /// Mount `build` in a fresh child owner of the flow's owner.
    fn mount(&self, rt: &Rc<Runtime>, build: impl FnOnce() -> View) -> Branch {
        self.mount_in(rt, self.parent.child(self.at), build)
    }

    /// Mount `build` in `owner`, a child owner the caller made.
    fn mount_in(&self, rt: &Rc<Runtime>, owner: Owner, build: impl FnOnce() -> View) -> Branch {
        mount_branch(rt, owner, self.frag, build)
    }

    /// Take `old` out of the slot as the presence mode says. Returns true when
    /// the new content must wait: `on_exit` is then the exit callback of the
    /// old branch.
    fn retire(
        &self,
        rt: &Rc<Runtime>,
        old: Option<Branch>,
        on_exit: impl FnOnce() -> Handler<()>,
    ) -> bool {
        let Some(branch) = old else {
            return false;
        };
        match self.presence {
            PresenceMode::Sync => {}
            PresenceMode::PopLayout => {
                ViewState::tree_mut(rt).set_exit(branch.node, Some(ExitLayout::Pop), None);
            }
            PresenceMode::Wait => {
                ViewState::tree_mut(rt).set_exit(branch.node, None, Some(on_exit()));
            }
        }
        unmount_branch(rt, branch);
        self.presence == PresenceMode::Wait
    }

    /// The exit callback of a waiting branch: calls `exited` while the
    /// flow's owner is alive.
    fn exit(&self, exited: impl Fn(&Rc<Runtime>) + 'static) -> Handler<()> {
        let rt = self.rt.clone();
        let parent = self.parent;
        Handler::from_fn(move |()| {
            if let Some(rt) = rt.upgrade()
                && parent.is_alive()
            {
                exited(&rt);
            }
        })
    }
}

// ---------------------------------------------------------------------------
// show

// xtask-allow(state-fields): The condition source and optional fallback are independent view construction inputs.
/// Conditional view. Built by [`show`].
pub struct Show {
    pub(crate) when: InputProp<bool>,
    pub(crate) child: Build,
    pub(crate) fallback: Option<Build>,
    pub(crate) presence: PresenceMode,
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
pub fn show<V: IntoView>(
    when: impl IntoReactive<bool>,
    child: impl crate::view::IntoViewFn<V>,
) -> Show {
    Show {
        when: when.into_prop(),
        child: build(child),
        fallback: None,
        presence: PresenceMode::Sync,
        at: Location::caller(),
    }
}

impl Show {
    /// Render `f()` while `when` is false.
    pub fn fallback<V: IntoView>(mut self, f: impl crate::view::IntoViewFn<V>) -> Self {
        self.fallback = Some(build(f));
        self
    }

    /// How the old content leaves when `when` changes (default `PresenceMode::Sync`).
    pub fn presence(mut self, mode: PresenceMode) -> Self {
        self.presence = mode;
        self
    }
}

/// Which arm of a `show` is wanted.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Next {
    Child,
    Fallback,
}

/// What a `show` has mounted.
enum ShowArm {
    /// Nothing: before the first run, and while the effect replaces the branch.
    Pending,
    /// The child, while the condition is true.
    Child(Branch),
    /// The fallback (if any), while the condition is false.
    Fallback(Option<Branch>),
    /// Wait mode: the old branch is leaving; `next` mounts when it has gone.
    Leaving { next: Next },
}

impl ShowArm {
    /// Whether this arm already shows `next`.
    fn shows(&self, next: Next) -> bool {
        matches!(
            (self, next),
            (ShowArm::Child(_), Next::Child) | (ShowArm::Fallback(_), Next::Fallback)
        )
    }

    fn into_branch(self) -> Option<Branch> {
        match self {
            ShowArm::Child(branch) => Some(branch),
            ShowArm::Fallback(branch) => branch,
            ShowArm::Pending | ShowArm::Leaving { .. } => None,
        }
    }
}

/// The state of one `show`: its arm and the builders it mounts.
struct ShowSlot {
    site: Site,
    child: Build,
    fallback: Option<Build>,
    arm: RefCell<ShowArm>,
}

impl ShowSlot {
    /// Build the arm for `next` now.
    fn mount(&self, rt: &Rc<Runtime>, next: Next) -> ShowArm {
        match next {
            Next::Child => ShowArm::Child(self.site.mount(rt, || self.child.call(()))),
            Next::Fallback => ShowArm::Fallback(
                self.fallback
                    .as_ref()
                    .map(|f| self.site.mount(rt, || f.call(()))),
            ),
        }
    }

    /// Show `next`. A branch that already shows it stays; a waiting one only
    /// records `next`.
    fn switch_to(self: &Rc<Self>, rt: &Rc<Runtime>, next: Next) {
        if let ShowArm::Leaving { next: wanted } = &mut *self.arm.borrow_mut() {
            *wanted = next;
            return;
        }
        if self.arm.borrow().shows(next) {
            return;
        }
        let old = std::mem::replace(&mut *self.arm.borrow_mut(), ShowArm::Pending);
        let slot = Rc::clone(self);
        let waits = self.site.retire(rt, old.into_branch(), || {
            self.site.exit(move |rt: &Rc<Runtime>| slot.exited(rt))
        });
        let arm = if waits {
            ShowArm::Leaving { next }
        } else {
            self.mount(rt, next)
        };
        *self.arm.borrow_mut() = arm;
    }

    /// The old branch of a waiting `show` has gone: mount what is wanted now.
    fn exited(&self, rt: &Rc<Runtime>) {
        let next = match &*self.arm.borrow() {
            ShowArm::Leaving { next } => *next,
            _ => return,
        };
        let arm = self.mount(rt, next);
        *self.arm.borrow_mut() = arm;
    }
}

impl IntoView for Show {
    fn into_view(self) -> View {
        let Show {
            when,
            child,
            fallback,
            presence,
            at,
        } = self;
        view_fn(move |cx| {
            let frag = cx.fragment();
            let slot = Rc::new(ShowSlot {
                site: Site {
                    rt: cx.weak(),
                    frag,
                    parent: cx.owner,
                    at,
                    presence,
                },
                child,
                fallback,
                arm: RefCell::new(ShowArm::Pending),
            });
            cx.effect(EffectKind::Internal, at, move || {
                let Some(rt) = slot.site.rt.upgrade() else {
                    return;
                };
                // The condition only computes a value: pure.
                let when = kivact_runtime::pure(|| when.get());
                slot.switch_to(&rt, if when { Next::Child } else { Next::Fallback });
            });
            frag
        })
    }
}

// ---------------------------------------------------------------------------
// show_some

// xtask-allow(state-fields): The optional-value source and optional fallback are independent view construction inputs.
/// View for `Some` values. Built by [`show_some`].
pub struct ShowSome<T: 'static> {
    pub(crate) value: InputProp<Option<T>>,
    pub(crate) child: crate::view::ViewFn<Value<T>>,
    pub(crate) fallback: Option<Build>,
    pub(crate) presence: PresenceMode,
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
        child: crate::view::ViewFn::from(child),
        fallback: None,
        presence: PresenceMode::Sync,
        at: Location::caller(),
    }
}

impl<T: 'static> ShowSome<T> {
    /// Render `f()` while the source is `None`.
    pub fn fallback<V: IntoView>(mut self, f: impl crate::view::IntoViewFn<V>) -> Self {
        self.fallback = Some(build(f));
        self
    }

    /// How the old content leaves when the source changes between `None` and
    /// `Some` (default `PresenceMode::Sync`).
    pub fn presence(mut self, mode: PresenceMode) -> Self {
        self.presence = mode;
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
    /// Wait mode: the old branch is leaving; `next` (the latest source value)
    /// mounts when it has gone.
    Leaving { next: Option<T> },
}

impl<T: 'static> SomeArm<T> {
    fn into_branch(self) -> Option<Branch> {
        match self {
            SomeArm::Child { branch, .. } => Some(branch),
            SomeArm::Fallback(branch) => branch,
            SomeArm::Pending | SomeArm::Leaving { .. } => None,
        }
    }
}

/// The state of one `show_some`: its arm and the builders it mounts.
struct SomeSlot<T: 'static> {
    site: Site,
    child: crate::view::ViewFn<Value<T>>,
    fallback: Option<Build>,
    arm: RefCell<SomeArm<T>>,
}

impl<T: Clone + 'static> SomeSlot<T> {
    /// Build the arm for `value` now.
    fn mount(&self, rt: &Rc<Runtime>, value: Option<T>) -> SomeArm<T> {
        match value {
            Some(v) => {
                let owner = self.site.parent.child(self.site.at);
                let signal = owner.signal(v, self.site.at);
                let branch = self
                    .site
                    .mount_in(rt, owner, || self.child.call(signal.into_value()));
                SomeArm::Child {
                    branch,
                    value: signal,
                }
            }
            None => SomeArm::Fallback(
                self.fallback
                    .as_ref()
                    .map(|f| self.site.mount(rt, || f.call(()))),
            ),
        }
    }

    /// Show `value`: Some→Some updates the signal of the branch in place; a
    /// waiting branch only records the latest value.
    fn switch_to(self: &Rc<Self>, rt: &Rc<Runtime>, value: Option<T>) {
        if let SomeArm::Leaving { next } = &mut *self.arm.borrow_mut() {
            *next = value;
            return;
        }
        let shown = match &*self.arm.borrow() {
            SomeArm::Child { value, .. } => Some(*value),
            _ => None,
        };
        match (value, shown) {
            // Some->Some keeps the branch and only updates its value.
            (Some(v), Some(signal)) => {
                signal.update(move |slot| *slot = v);
            }
            (None, _) if matches!(*self.arm.borrow(), SomeArm::Fallback(_)) => {}
            (value, _) => self.replace(rt, value),
        }
    }

    /// Retire the arm shown now and show `value` (after the exit in `Wait`).
    fn replace(self: &Rc<Self>, rt: &Rc<Runtime>, value: Option<T>) {
        let old = std::mem::replace(&mut *self.arm.borrow_mut(), SomeArm::Pending);
        let slot = Rc::clone(self);
        let waits = self.site.retire(rt, old.into_branch(), || {
            self.site.exit(move |rt: &Rc<Runtime>| slot.exited(rt))
        });
        let arm = if waits {
            SomeArm::Leaving { next: value }
        } else {
            self.mount(rt, value)
        };
        *self.arm.borrow_mut() = arm;
    }

    /// The old branch of a waiting `show_some` has gone: mount what is wanted now.
    fn exited(&self, rt: &Rc<Runtime>) {
        let next = match &*self.arm.borrow() {
            SomeArm::Leaving { next } => next.clone(),
            _ => return,
        };
        let arm = self.mount(rt, next);
        *self.arm.borrow_mut() = arm;
    }
}

impl<T: Clone + 'static> IntoView for ShowSome<T> {
    fn into_view(self) -> View {
        let ShowSome {
            value,
            child,
            fallback,
            presence,
            at,
        } = self;
        view_fn(move |cx| {
            let frag = cx.fragment();
            let slot = Rc::new(SomeSlot {
                site: Site {
                    rt: cx.weak(),
                    frag,
                    parent: cx.owner,
                    at,
                    presence,
                },
                child,
                fallback,
                arm: RefCell::new(SomeArm::Pending),
            });
            cx.effect(EffectKind::Internal, at, move || {
                let Some(rt) = slot.site.rt.upgrade() else {
                    return;
                };
                let v = kivact_runtime::pure(|| value.get());
                slot.switch_to(&rt, v);
            });
            frag
        })
    }
}

// ---------------------------------------------------------------------------
// dynamic

/// A view rebuilt in a fresh branch each time its source changes. Built by
/// [`dynamic`]; it is a view (`IntoView`).
pub struct Dynamic<T: 'static> {
    source: InputProp<T>,
    child: crate::view::ViewFn<T>,
    presence: PresenceMode,
    at: &'static Location<'static>,
}

impl<T: 'static> Dynamic<T> {
    /// How the old content leaves when the source changes (default `PresenceMode::Sync`).
    pub fn presence(mut self, mode: PresenceMode) -> Self {
        self.presence = mode;
        self
    }
}

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
pub fn dynamic<T: PropValue, V: IntoView>(
    source: impl IntoReactive<T>,
    child: impl Fn(T) -> V + 'static,
) -> Dynamic<T> {
    Dynamic {
        source: source.into_prop(),
        child: crate::view::ViewFn::from(child),
        presence: PresenceMode::Sync,
        at: Location::caller(),
    }
}

/// What a `dynamic` has mounted.
enum DynamicArm<T> {
    Pending,
    /// The branch, and the value it was built from.
    Mounted {
        value: T,
        branch: Branch,
    },
    /// Wait mode: the old branch is leaving; `next` (the latest source value)
    /// mounts when it has gone.
    Leaving {
        next: T,
    },
}

/// The state of one `dynamic`: its arm and the builder it mounts.
struct DynamicSlot<T: 'static> {
    site: Site,
    child: crate::view::ViewFn<T>,
    arm: RefCell<DynamicArm<T>>,
}

impl<T: PropValue> DynamicSlot<T> {
    /// Build the branch for `value` now.
    fn mount(&self, rt: &Rc<Runtime>, value: T) -> DynamicArm<T> {
        let built = value.clone();
        let branch = self.site.mount(rt, || self.child.call(built));
        DynamicArm::Mounted { value, branch }
    }

    /// Show `value`: an equal value keeps the branch; a waiting branch only
    /// records the latest value.
    fn switch_to(self: &Rc<Self>, rt: &Rc<Runtime>, value: T) {
        if let DynamicArm::Leaving { next } = &mut *self.arm.borrow_mut() {
            *next = value;
            return;
        }
        let unchanged = matches!(
            &*self.arm.borrow(),
            DynamicArm::Mounted { value: built, .. } if *built == value
        );
        if unchanged {
            return;
        }
        let old = std::mem::replace(&mut *self.arm.borrow_mut(), DynamicArm::Pending);
        let branch = match old {
            DynamicArm::Mounted { branch, .. } => Some(branch),
            DynamicArm::Pending | DynamicArm::Leaving { .. } => None,
        };
        let slot = Rc::clone(self);
        let waits = self.site.retire(rt, branch, || {
            self.site.exit(move |rt: &Rc<Runtime>| slot.exited(rt))
        });
        let arm = if waits {
            DynamicArm::Leaving { next: value }
        } else {
            self.mount(rt, value)
        };
        *self.arm.borrow_mut() = arm;
    }

    /// The old branch of a waiting `dynamic` has gone: mount what is wanted now.
    fn exited(&self, rt: &Rc<Runtime>) {
        let next = match &*self.arm.borrow() {
            DynamicArm::Leaving { next } => next.clone(),
            _ => return,
        };
        let arm = self.mount(rt, next);
        *self.arm.borrow_mut() = arm;
    }
}

impl<T: PropValue> IntoView for Dynamic<T> {
    fn into_view(self) -> View {
        let Dynamic {
            source,
            child,
            presence,
            at,
        } = self;
        view_fn(move |cx| {
            let frag = cx.fragment();
            let slot = Rc::new(DynamicSlot {
                site: Site {
                    rt: cx.weak(),
                    frag,
                    parent: cx.owner,
                    at,
                    presence,
                },
                child,
                arm: RefCell::new(DynamicArm::Pending),
            });
            cx.effect(EffectKind::Internal, at, move || {
                let Some(rt) = slot.site.rt.upgrade() else {
                    return;
                };
                let value = kivact_runtime::pure(|| source.get());
                slot.switch_to(&rt, value);
            });
            frag
        })
    }
}
