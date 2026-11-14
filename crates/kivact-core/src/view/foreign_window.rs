//! Role: The `foreign_window` view: a window whose whole content area is a
//! native child window for UI another library draws.

use std::cell::{Cell, RefCell};
use std::future::Future;
use std::marker::PhantomData;
use std::panic::Location;
use std::rc::Rc;

use kivact_runtime::{
    Binding, Completion, ErrorKind, Handler, HandlerResult, IntoBinding, IntoText, IntoValue,
    Owner, Runtime, UiError, Value, require_effectful,
};
use kivact_types::{
    DismissReason, NativeWindow, PlatformRequest, RequestId, Size, SurfaceSize, WindowId,
    WindowMode, WindowPlacement,
};

use crate::geometry;
use crate::view::descriptor::{IntoView, View, view_fn};
use crate::view::foreign::{ForeignChild, ForeignHooks, ForeignPhase};
use crate::view::state::{ForeignSizeRequest, ViewState};

/// An OS window of this App whose whole content area is a native child window
/// for UI another library draws, such as a plugin editor (CLAP, VST3).
///
/// Like [`window`](crate::view::window): while `open` is true the window
/// exists; writing false closes it; the title bar's close calls `on_dismiss`
/// and, unless it is cancelled, writes `open` false; it closes with the owner
/// that declared it and when the App ends. Kivact draws only the theme
/// background behind the child. The child always fills the content area and
/// follows its size. Detach the foreign UI in `on_destroy`: it runs before the
/// child and its window are destroyed, whichever way the window closes
/// (except after a platform failure or a panic).
///
/// Windows (Win32) and Linux X11. Elsewhere (Wayland, an embedded App, the
/// test host without `RenderOptions::foreign_windows`) it reports
/// `ErrorKind::Unsupported` once and stays closed; `open` is not changed.
#[track_caller]
pub fn foreign_window(title: impl IntoText, open: impl IntoBinding<bool>) -> ForeignWindow {
    ForeignWindow {
        title: title.into_text_value(),
        open: open.into_binding(),
        size: Size::new(640.0, 480.0),
        min_size: None,
        mode: WindowMode::Normal,
        placement: WindowPlacement::Auto,
        resizable: true.into_value(),
        dismiss: None,
        ready: None,
        resize: None,
        destroy: None,
        at: Location::caller(),
    }
}

/// The declaration of one foreign window, built by [`foreign_window`].
pub struct ForeignWindow {
    title: Value<String>,
    open: Binding<bool>,
    size: Size,
    // xtask-allow(state-fields): option: read once when the window opens; absent means the engine's minimum
    min_size: Option<Size>,
    mode: WindowMode,
    placement: WindowPlacement,
    resizable: Value<bool>,
    dismiss: Option<Handler<DismissReason>>,
    ready: Option<Handler<ForeignChild>>,
    resize: Option<Handler<SurfaceSize>>,
    destroy: Option<Box<dyn Fn()>>,
    at: &'static Location<'static>,
}

impl ForeignWindow {
    /// First content size in logical pixels (default 640 x 480), checked like
    /// `window`.
    pub fn size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.size = Size::new(width.into(), height.into());
        self
    }

    /// Smallest logical size the user can resize it to.
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

    /// Whether the user can resize the window (reactive, default true). Set
    /// false for an editor that cannot resize; `ForeignChild::request_size`
    /// still works.
    pub fn resizable(mut self, resizable: impl IntoValue<bool>) -> Self {
        self.resizable = resizable.into_value();
        self
    }

    /// The user asked to close the window; same rules as `window`'s
    /// `on_dismiss`.
    pub fn on_dismiss<R: HandlerResult>(
        mut self,
        f: impl Fn(DismissReason) -> R + 'static,
    ) -> Self {
        self.dismiss = Some(Handler::from_fn(f));
        self
    }

    /// The window and its child exist (once per opening). Open the foreign UI
    /// with `child.window()` as its parent.
    pub fn on_ready<R: HandlerResult>(mut self, f: impl Fn(ForeignChild) -> R + 'static) -> Self {
        self.ready = Some(Handler::from_fn(f));
        self
    }

    /// The child's physical size or scale changed (the user resized the
    /// window, it moved to a monitor with another scale, or a `request_size`
    /// was applied).
    pub fn on_resize<R: HandlerResult>(mut self, f: impl Fn(SurfaceSize) -> R + 'static) -> Self {
        self.resize = Some(Handler::from_fn(f));
        self
    }

    /// Runs once per opening, before the child and the window are destroyed.
    pub fn on_destroy(mut self, f: impl Fn() + 'static) -> Self {
        self.destroy = Some(Box::new(f));
        self
    }
}

impl IntoView for ForeignWindow {
    fn into_view(self) -> View {
        if let Err(error) = geometry::check_window_sizes(
            Some(self.size),
            self.min_size,
            ("ForeignWindow::size", "ForeignWindow::min_size"),
            self.at,
        ) {
            return view_fn(move |cx| {
                cx.rt.report(Some(cx.owner), error);
                cx.fragment()
            });
        }
        let hooks = Rc::new(ForeignHooks {
            title: self.title,
            open: self.open,
            size: self.size,
            min_size: self.min_size,
            mode: self.mode,
            placement: self.placement,
            resizable: self.resizable,
            dismiss: self.dismiss,
            ready: RefCell::new(self.ready),
            resize: RefCell::new(self.resize),
            destroy: RefCell::new(self.destroy),
            phase: Cell::new(ForeignPhase::Waiting),
        });
        crate::view::window_view::foreign_window_view(hooks, self.at)
    }
}

impl ForeignChild {
    /// The raw handle to give the foreign UI as its parent.
    pub fn window(&self) -> NativeWindow {
        self.child
    }

    /// The child's size when it became ready (later sizes arrive in
    /// `on_resize`).
    pub fn size(&self) -> SurfaceSize {
        self.size
    }

    /// Resize the window so the child is `width` x `height` physical pixels
    /// (the size the foreign UI asked for). Same contract as
    /// [`platform::request_window_size`](crate::platform::request_window_size)
    /// (clamped to `min_size`; a refusal is a platform error), but for this
    /// window and in physical pixels. After the window closed it resolves to
    /// `ErrorKind::Closed`.
    #[track_caller]
    pub fn request_size(
        &self,
        width: u32,
        height: u32,
    ) -> impl Future<Output = Result<(), UiError>> + 'static {
        let at = Location::caller();
        let slot = self.start_request_size(width, height, at);
        async move {
            match slot {
                Ok(slot) => slot.await,
                Err(error) => Err(error),
            }
        }
    }

    /// Send the window size request; the answer is completed by the host's
    /// effect result or by the window's close.
    fn start_request_size(
        &self,
        width: u32,
        height: u32,
        at: &'static Location<'static>,
    ) -> Result<Completion<Result<(), UiError>>, UiError> {
        let owner = Owner::ambient("ForeignChild::request_size", at);
        let rt = owner.runtime();
        let Some(state) = ViewState::of(&rt).window_record(self.window) else {
            return Err(UiError::new(
                ErrorKind::Closed,
                "the foreign window is closed",
                Some(at),
            ));
        };
        if width == 0 || height == 0 {
            return Err(UiError::new(
                ErrorKind::Message,
                "ForeignChild::request_size dimensions must be physical pixels of at least 1",
                Some(at),
            ));
        }
        require_effectful("ForeignChild::request_size");
        let reported = ViewState::of(&rt)
            .env(self.window)
            .map_or(self.size.scale_factor, |env| env.environment().scale_factor);
        let scale = if reported.is_finite() && reported > 0.0 {
            reported
        } else {
            1.0
        };
        let mut size = Size {
            width: f64::from(width) / scale,
            height: f64::from(height) / scale,
        };
        if let Some(min) = state.min_size {
            size.width = size.width.max(min.width);
            size.height = size.height.max(min.height);
        }
        if !kivact_types::env::is_valid_window_size(size) {
            return Err(UiError::new(
                ErrorKind::Message,
                "ForeignChild::request_size dimensions must fit the native window extent",
                Some(at),
            ));
        }
        let slot = Completion::new();
        let id = {
            let mut requests = ViewState::of(&rt).requests.borrow_mut();
            let id = requests.request_id();
            requests.foreign_sizes.insert(
                id,
                ForeignSizeRequest {
                    window: self.window,
                    at,
                    slot: slot.clone(),
                },
            );
            id
        };
        ViewState::tree(&rt).engine().effect(
            RequestId(id),
            kivact_engine::contract::HostRequest::Platform {
                window: self.window,
                request: PlatformRequest::RequestWindowSize(size),
            },
        );
        rt.request_turn();
        Ok(slot)
    }
}

/// Deliver a ready or resize report to window `window`'s declaration.
pub(crate) fn changed(
    rt: &Runtime,
    window: WindowId,
    change: &kivact_engine::contract::ForeignChange,
) {
    let Some(record) = ViewState::of(rt).window_record(window) else {
        return;
    };
    let Some(hooks) = record.foreign.clone() else {
        return;
    };
    match change {
        kivact_engine::contract::ForeignChange::Ready { child, size } => {
            if hooks.phase.get() != ForeignPhase::Waiting {
                return;
            }
            let child = ForeignChild {
                window,
                child: *child,
                size: *size,
                _thread: PhantomData,
            };
            hooks.phase.set(ForeignPhase::Ready(child));
            hooks.ready(rt, record.declared_in, child);
        }
        kivact_engine::contract::ForeignChange::Resized(size) => {
            let ForeignPhase::Ready(child) = hooks.phase.get() else {
                return;
            };
            hooks.phase.set(ForeignPhase::Ready(ForeignChild {
                size: *size,
                ..child
            }));
            hooks.resized(rt, record.declared_in, *size);
        }
    }
}
