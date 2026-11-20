//! Role: The `window` and `foreign_window` view: a portal into another OS
//! window of the App, whose content Kivact draws or a native child fills.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::{Rc, Weak};

use kivact_runtime::{
    EffectKind, ErrorKind, Handler, HandlerResult, IntoText, Owner, Runtime, Signal, UiError, Value,
};
use kivact_types::{
    DismissReason, Environment, NodeId, Size, WindowContent, WindowId, WindowMode, WindowPlacement,
    WindowRequest,
};

use crate::geometry;
use crate::view::descriptor::{IntoView, IntoViewFn, View, ViewFn, view_fn};
use crate::view::foreign::{ForeignHooks, ForeignPhase};
use crate::view::root::mount_window_root;
use crate::view::state::{EnvState, FocusKeys, ViewState, WindowDecl, WindowRecord, WindowScope};

/// A window of this App: while `open` is true its content is mounted and
/// shown in its own OS window. The content inherits the contexts of the
/// place where `window` is declared, like a React portal.
#[track_caller]
pub fn window(title: impl IntoText, open: Signal<bool>) -> Window {
    Window {
        decl: WindowDecl {
            title: title.into_text_value(),
            open: open,
            size: Size::new(640.0, 480.0),
            min_size: None,
            mode: WindowMode::Normal,
            placement: WindowPlacement::Auto,
            dismiss: None,
            foreign: None,
        },
        content: SlotContent::Views(None),
        at: Location::caller(),
    }
}

/// The description of one window, built by [`window`].
pub struct Window {
    pub(super) decl: WindowDecl,
    pub(super) content: SlotContent,
    pub(super) at: &'static Location<'static>,
}

/// What a window declaration contains.
pub(super) enum SlotContent {
    /// The Kivact views of a `window`, built again at each opening.
    Views(Option<ViewFn<()>>),
    /// The native child window machinery of a `foreign_window`; its hooks are
    /// the declaration's `foreign`.
    Foreign,
}

impl Window {
    /// The content, built again each time the window opens (API-6 factory).
    pub fn content<V: IntoView>(mut self, content: impl IntoViewFn<V>) -> Self {
        self.content = SlotContent::Views(Some(content.into_view_fn()));
        self
    }

    /// The size the window opens with, in logical pixels (default 640 × 480).
    pub fn size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.decl.size = Size::new(width.into(), height.into());
        self
    }

    /// The smallest size the user can resize the window to.
    pub fn min_size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.decl.min_size = Some(Size::new(width.into(), height.into()));
        self
    }

    /// What kind of window it is (structural, default `WindowMode::Normal`).
    pub fn mode(mut self, mode: WindowMode) -> Self {
        self.decl.mode = mode;
        self
    }

    /// Where it opens relative to the window it is declared in (structural,
    /// default `WindowPlacement::Auto`).
    pub fn placement(mut self, placement: WindowPlacement) -> Self {
        self.decl.placement = placement;
        self
    }

    /// Called when the user asks to close the window (title bar button,
    /// Alt+F4). Return `()` (or `Handled::No`) to let it close: `open` is set
    /// to false. `Handled::Yes` and `Handled::PreventDefault` keep it open, as
    /// they skip every default action; so does a failed handler (its error is
    /// reported). Unlike an overlay, a window closes itself, like `dialog`.
    pub fn on_dismiss<R: HandlerResult>(
        mut self,
        f: impl Fn(DismissReason) -> R + 'static,
    ) -> Self {
        self.decl.dismiss = Some(Handler::from_fn(f));
        self
    }
}

/// Which arm of a `window` is mounted.
#[derive(Clone, Copy, PartialEq, Eq)]
enum WindowArm {
    Closed,
    Open { id: WindowId },
}

/// The retained state of one `window`: its declaration site and arm.
struct WindowSlot {
    rt: Weak<Runtime>,
    parent: kivact_runtime::Owner,
    parent_window: WindowId,
    at: &'static Location<'static>,
    arm: RefCell<WindowArm>,
    unsupported: kivact_types::cell::ReportOnce,
    decl: Rc<WindowDecl>,
    content: SlotContent,
}

impl WindowSlot {
    /// Show the window when `open`, hide it otherwise.
    fn switch(
        self: &Rc<Self>,
        rt: &Rc<Runtime>,
        open: bool,
        capabilities: kivact_types::Capabilities,
    ) {
        let arm = *self.arm.borrow();
        match (arm, open) {
            (WindowArm::Open { id }, false) => self.close(rt, id, true),
            (WindowArm::Open { .. }, true) | (WindowArm::Closed, false) => {}
            (WindowArm::Closed, true) => self.open(rt, capabilities),
        }
    }

    /// Open the window if the platform can hold it; report it once if not.
    fn open(self: &Rc<Self>, rt: &Rc<Runtime>, capabilities: kivact_types::Capabilities) {
        if !self.supports(capabilities) {
            return;
        }
        let (id, owner, title) = self.begin(rt);
        let root = match &self.decl.foreign {
            Some(hooks) => self.mount_foreign(rt, id, owner, title, hooks),
            None => self.mount_views(rt, id, owner),
        };
        self.watch(rt, owner, id, title);
        debug_assert_eq!(ViewState::tree(rt).root(id), Some(root));
        *self.arm.borrow_mut() = WindowArm::Open { id };
    }

    /// Whether the platform can open this declaration; report it once if not.
    fn supports(&self, capabilities: kivact_types::Capabilities) -> bool {
        let supported = match &self.content {
            SlotContent::Views(_) => capabilities.multi_window,
            SlotContent::Foreign => capabilities.foreign_window,
        };
        if !supported && self.unsupported.note() {
            let message = match &self.content {
                SlotContent::Views(_) => "this platform opens no window beyond the first",
                SlotContent::Foreign => "this platform opens no window with a native child window",
            };
            self.parent
                .report(UiError::new(ErrorKind::Unsupported, message, Some(self.at)));
        }
        supported
    }

    /// Record the window's owner, environment and open request; the host
    /// opens the OS window after the turn.
    fn begin(&self, rt: &Rc<Runtime>) -> (WindowId, Owner, Value<String>) {
        let state = ViewState::of(rt);
        let id = state.next_window();
        let owner = self.parent.child(self.at);
        let parent_env = state
            .env(self.parent_window)
            .expect("invariant: the parent window is open");
        let decl = self.decl.clone();
        let content = match &self.content {
            SlotContent::Views(_) => WindowContent::Drawn,
            SlotContent::Foreign => WindowContent::Foreign,
        };
        ViewState::tree(rt).engine().open_window(WindowRequest {
            id,
            title: decl.title.get_untracked(),
            size: decl.size,
            min_size: decl.min_size,
            mode: decl.mode,
            placement: decl.placement,
            parent: Some(self.parent_window),
            content,
        });
        state.install_window(
            id,
            open_window_record(
                id,
                self.parent,
                owner,
                decl.clone(),
                parent_env.environment(),
                parent_env.surface_active(),
                self.at,
            ),
        );
        (id, owner, decl.title)
    }

    /// Mount a `window`'s content as the window root.
    fn mount_views(self: &Rc<Self>, rt: &Rc<Runtime>, id: WindowId, owner: Owner) -> NodeId {
        let content = match &self.content {
            SlotContent::Views(content) => content.clone(),
            SlotContent::Foreign => None,
        };
        mount_window_root(
            rt,
            id,
            owner,
            Box::new(move || match content {
                Some(content) => content.call(()),
                None => View::empty(),
            }),
            self.at,
        )
    }

    /// Mount the empty themed root of a `foreign_window` and register its
    /// cleanup.
    fn mount_foreign(
        self: &Rc<Self>,
        rt: &Rc<Runtime>,
        id: WindowId,
        owner: Owner,
        title: Value<String>,
        hooks: &Rc<ForeignHooks>,
    ) -> NodeId {
        hooks.phase.set(ForeignPhase::Waiting);
        let cleanup = hooks.clone();
        let cleanup_owner = owner;
        owner.on_cleanup(move || {
            cleanup_owner.run_handler(|| cleanup.release());
        });
        let label = title.get_untracked();
        mount_window_root(
            rt,
            id,
            owner,
            Box::new(move || {
                crate::view::elements::node()
                    .role(kivact_types::Role::Group)
                    .name(label)
                    .into_view()
            }),
            self.at,
        )
    }

    /// Keep the title, minimum size and resizable flag of `id` in sync.
    fn watch(self: &Rc<Self>, rt: &Rc<Runtime>, owner: Owner, id: WindowId, title: Value<String>) {
        let min_size = self.decl.min_size;
        let resizable = self.decl.foreign.as_ref().map(|hooks| hooks.resizable);
        let weak = Rc::downgrade(rt);
        owner.effect(
            EffectKind::Render,
            move || {
                let Some(rt) = weak.upgrade() else { return };
                let title = title.get();
                let resizable = resizable.is_none_or(|resizable| resizable.get());
                ViewState::tree(&rt)
                    .engine()
                    .set_window(id, title, min_size, resizable);
            },
            self.at,
        );
    }

    /// Close `id`: remove its subtree, close the OS window and drop its
    /// record. `dispose` is false when the declaring owner is being disposed
    /// (the window's owner goes with it).
    fn close(self: &Rc<Self>, rt: &Rc<Runtime>, id: WindowId, dispose: bool) {
        let root = ViewState::tree(rt).root(id);
        if let Some(root) = root {
            let owner = ViewState::tree(rt).owner_of(root);
            let removed = ViewState::tree_mut(rt).remove(root);
            removed.commit(rt);
            ViewState::tree(rt).engine().close_window(id);
            if dispose && owner.is_alive() {
                owner.dispose();
            }
        } else {
            ViewState::tree(rt).engine().close_window(id);
        }
        ViewState::of(rt).remove_window(id);
        ViewState::of(rt).close_foreign_sizes(id);
        *self.arm.borrow_mut() = WindowArm::Closed;
    }
}

/// Record `window`, declared by `declared_in` with `decl`: its owner scope,
/// environment, focus and pointer sources. Every window, the main one
/// included, is recorded here.
pub(crate) fn open_window_record(
    window: WindowId,
    declared_in: Owner,
    owner: Owner,
    decl: Rc<WindowDecl>,
    env: Environment,
    surface_active: bool,
    at: &'static Location<'static>,
) -> WindowRecord {
    owner.provide(WindowScope(window));
    WindowRecord {
        declared_in,
        decl,
        env: Rc::new(EnvState::new(owner, env, surface_active, at)),
        focus: FocusKeys::new(owner, at),
        hover: owner.signal(None, at),
        mouse: owner.signal(None, at),
    }
}

impl IntoView for Window {
    fn into_view(self) -> View {
        let Window { decl, content, at } = self;
        if let Err(error) = geometry::check_window_sizes(
            Some(decl.size),
            decl.min_size,
            ("Window::size", "Window::min_size"),
            at,
        ) {
            return view_fn(move |cx| {
                cx.rt.report(Some(cx.owner), error);
                cx.fragment()
            });
        }
        view_fn(move |cx| {
            let frag = cx.fragment();
            let slot = Rc::new(WindowSlot {
                rt: cx.weak(),
                parent: cx.owner,
                parent_window: cx.window,
                at,
                arm: RefCell::new(WindowArm::Closed),
                unsupported: kivact_types::cell::ReportOnce::new(),
                decl: Rc::new(decl),
                content,
            });
            cx.owner.store("WindowSlot", slot.clone(), at);
            let cleanup = slot.clone();
            cx.owner.on_cleanup(move || {
                let WindowArm::Open { id } = *cleanup.arm.borrow() else {
                    return;
                };
                if let Some(rt) = cleanup.rt.upgrade() {
                    cleanup.close(&rt, id, false);
                }
            });
            let capabilities = ViewState::of(cx.rt).capabilities;
            cx.effect(EffectKind::Internal, at, move || {
                let Some(rt) = slot.rt.upgrade() else {
                    return;
                };
                let (wanted, capabilities) =
                    kivact_runtime::pure(|| (slot.decl.open.get(), capabilities.get_untracked()));
                slot.switch(&rt, wanted, capabilities);
            });
            frag
        })
    }
}
