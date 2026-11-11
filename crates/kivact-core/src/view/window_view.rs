//! Role: The `window` view: a portal into another OS window of the App.

use std::cell::{Cell, RefCell};
use std::panic::Location;
use std::rc::{Rc, Weak};

use kivact_runtime::{
    Binding, EffectKind, ErrorKind, Handler, HandlerResult, IntoBinding, IntoText, Runtime,
    UiError, Value,
};
use kivact_types::{DismissReason, Size, WindowId, WindowMode, WindowPlacement, WindowRequest};

use crate::geometry;
use crate::view::descriptor::{IntoView, IntoViewFn, View, ViewFn, view_fn};
use crate::view::root::mount_window_root;
use crate::view::state::{
    EnvState, FocusKeys, PointerObservation, ViewState, WindowRecord, WindowScope,
};

/// A window of this App: while `open` is true its content is mounted and
/// shown in its own OS window. The content inherits the contexts of the
/// place where `window` is declared, like a React portal.
#[track_caller]
pub fn window(title: impl IntoText, open: impl IntoBinding<bool>) -> Window {
    Window {
        title: title.into_text_value(),
        open: open.into_binding(),
        content: None,
        size: Size::new(640.0, 480.0),
        min_size: None,
        mode: WindowMode::Normal,
        placement: WindowPlacement::Auto,
        dismiss: None,
        at: Location::caller(),
    }
}

/// The description of one window, built by [`window`].
pub struct Window {
    title: Value<String>,
    open: Binding<bool>,
    content: Option<ViewFn<()>>,
    size: Size,
    // xtask-allow(state-fields): option: read once when the window opens; absent means the engine's minimum
    min_size: Option<Size>,
    mode: WindowMode,
    placement: WindowPlacement,
    dismiss: Option<Handler<DismissReason>>,
    at: &'static Location<'static>,
}

impl Window {
    /// The content, built again each time the window opens (API-6 factory).
    pub fn content<V: IntoView>(mut self, content: impl IntoViewFn<V>) -> Self {
        self.content = Some(content.into_view_fn());
        self
    }

    /// The size the window opens with, in logical pixels (default 640 × 480).
    pub fn size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.size = Size::new(width.into(), height.into());
        self
    }

    /// The smallest size the user can resize the window to.
    pub fn min_size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.min_size = Some(Size::new(width.into(), height.into()));
        self
    }

    /// What kind of window it is (structural, default `WindowMode::Normal`).
    pub fn mode(mut self, mode: WindowMode) -> Self {
        self.mode = mode;
        self
    }

    /// Where it opens relative to the window it is declared in (structural,
    /// default `WindowPlacement::Auto`).
    pub fn placement(mut self, placement: WindowPlacement) -> Self {
        self.placement = placement;
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
        self.dismiss = Some(Handler::from_fn(f));
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
    title: Value<String>,
    open: Binding<bool>,
    content: Option<ViewFn<()>>,
    size: Size,
    min_size: Option<Size>,
    mode: WindowMode,
    placement: WindowPlacement,
    dismiss: Option<Handler<DismissReason>>,
}

impl WindowSlot {
    /// Show the window when `open`, hide it otherwise.
    fn switch(self: &Rc<Self>, rt: &Rc<Runtime>, open: bool, multi_window: bool) {
        let arm = *self.arm.borrow();
        match (arm, open) {
            (WindowArm::Open { id }, false) => self.close(rt, id, true),
            (WindowArm::Open { .. }, true) | (WindowArm::Closed, false) => {}
            (WindowArm::Closed, true) => self.open(rt, multi_window),
        }
    }

    fn open(self: &Rc<Self>, rt: &Rc<Runtime>, multi_window: bool) {
        let state = ViewState::of(rt);
        if !multi_window {
            if self.unsupported.note() {
                self.parent.report(UiError::new(
                    ErrorKind::Unsupported,
                    "this platform opens no window beyond the first",
                    Some(self.at),
                ));
            }
            return;
        }
        let id = state.next_window();
        let owner = self.parent.child(self.at);
        let parent_env = state
            .env(self.parent_window)
            .expect("invariant: the parent window is open");
        let env = Rc::new(EnvState::new(
            owner,
            parent_env.environment(),
            parent_env.surface_active(),
            self.at,
        ));
        let focus = FocusKeys::new(owner, self.at);
        let mouse = owner.signal(None, self.at);
        let title = self.title;
        let min_size = self.min_size;
        ViewState::tree(rt).engine().open_window(WindowRequest {
            id,
            title: title.get_untracked(),
            size: self.size,
            min_size,
            mode: self.mode,
            placement: self.placement,
            parent: Some(self.parent_window),
        });
        owner.provide(WindowScope(id));
        state.install_window(
            id,
            WindowRecord {
                declared_in: self.parent,
                env,
                focus,
                hover: Cell::new(PointerObservation::Plain(None)),
                mouse: Cell::new(PointerObservation::Reactive(mouse)),
                title,
                min_size,
                dismiss: self.dismiss.clone(),
                open: self.open,
                mode: self.mode,
            },
        );
        let content = self.content.clone();
        let root = mount_window_root(
            rt,
            id,
            owner,
            Box::new(move || match content {
                Some(content) => content.call(()),
                None => View::empty(),
            }),
            self.at,
        );
        let weak = Rc::downgrade(rt);
        owner.effect(
            EffectKind::Render,
            move || {
                let Some(rt) = weak.upgrade() else { return };
                let title = title.get();
                ViewState::tree(&rt)
                    .engine()
                    .set_window(id, title, min_size);
            },
            self.at,
        );
        debug_assert_eq!(ViewState::tree(rt).root(id), Some(root));
        *self.arm.borrow_mut() = WindowArm::Open { id };
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
        *self.arm.borrow_mut() = WindowArm::Closed;
    }
}

impl IntoView for Window {
    fn into_view(self) -> View {
        let Window {
            title,
            open,
            content,
            size,
            min_size,
            mode,
            placement,
            dismiss,
            at,
        } = self;
        if let Err(error) = geometry::check_window_sizes(
            Some(size),
            min_size,
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
                title,
                open,
                content,
                size,
                min_size,
                mode,
                placement,
                dismiss,
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
                let (wanted, multi_window) = kivact_runtime::pure(|| {
                    (open.get(), capabilities.get_untracked().multi_window)
                });
                slot.switch(&rt, wanted, multi_window);
            });
            frag
        })
    }
}
