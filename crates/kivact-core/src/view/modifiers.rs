//! Role: These helpers hold the logic the modifier methods share.
//!
//! The modifier methods are written once, as plain default methods:
//! the everyday set (`common`, `draw`, `intrinsic`) on
//! `kivact::control::NodeBuilder` and the advanced set (`advanced`,
//! `draw_over`) on `kivact::primitive::NodeExt`. `Node`, `Overlay`,
//! `VirtualList` and every component holding a root `Node` implement
//! `NodeBuilder`. `Part` has its own inherent methods for the semantic and
//! input subset, which forward to the same `Common` methods below. Adding a
//! modifier means writing the method on the trait, plus, when it has logic
//! beyond a field write, one `Common` method that `Part` shares.
//!
//! Modifiers transport properties through the builder's `Common`; `with`
//! additionally merges a part's semantic sources, constraints and defaults.
//! Their mounted meaning is decided by `mount::mount_host`.
//!
//! Handler layering (see `handler_layers` for the delivery order):
//! - `on_key`, `on_key_up`, `on_pointer` and `on_wheel` are `Layer::Caller`
//!   entries, the latest registration first: `No` passes the event to the
//!   next handler, `Yes` stops it, and an `Err` stops the rest and is
//!   reported once for the event.
//! - `on_*_default` is the component author's hook: a `Layer::NodeDefault`
//!   entry that runs after the callers the caller registered.
//! - `on_file` keeps one handler; registering it again replaces it.
//! - `on_press` uses the root's one Activate slot (pointer, Enter/Space and
//!   the accessibility Activate); registering it again replaces the caller's
//!   handler in that slot.
//! - `on_a11y` keeps one handler per action and a new registration replaces
//!   it.
//!
//! The extra semantics (the value range, `live_region`,
//! `sort`, `orientation`, `active_descendant`, `level`, `set_size`,
//! `pos_in_set`, `multiselectable` and the relations `described_by`,
//! `controls`, `labelled_by`) share one rarely allocated box of sources on
//! the node (`SemanticSources`), so an ordinary node stores nothing extra
//! for them; `mount::semantics` publishes them.

use std::rc::Rc;

use crate::view::common::{Common, SemanticSources};
use crate::view::input::{KeyEvent, KeyHandlerResult, PointerEvent, WheelEvent};
use crate::view::{A11yAction, A11yEvent, A11yRequest, HandlerResult};

/// The shared bodies of the modifiers that are more than a field write.
/// `NodeBuilder`, `NodeExt` and `Part` forward to them.
impl Common {
    /// The rarely allocated box of extra semantic sources.
    pub(crate) fn semantics_mut(&mut self) -> &mut SemanticSources {
        self.semantics.get_or_insert_with(Default::default)
    }

    /// `on_key`: run before the key handlers already registered.
    pub(crate) fn prepend_key<F, R>(&mut self, f: F)
    where
        F: Fn(KeyEvent) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.handlers.key.caller(move |e| f(e).into_handled());
    }

    /// `on_key_up`: run before the key release handlers already registered.
    pub(crate) fn prepend_key_up<F, R>(&mut self, f: F)
    where
        F: Fn(KeyEvent) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.handlers.key_up.caller(move |e| f(e).into_handled());
    }

    /// `on_pointer`: run before the pointer handlers already registered.
    pub(crate) fn prepend_pointer<F, R>(&mut self, f: F)
    where
        F: Fn(PointerEvent) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.handlers.pointer.caller(move |e| f(e).into_handled());
    }

    /// `on_wheel`: run before the wheel handlers already registered.
    pub(crate) fn prepend_wheel<F, R>(&mut self, f: F)
    where
        F: Fn(WheelEvent) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.handlers.wheel.caller(move |e| f(e).into_handled());
    }

    /// `on_ime`: run before the IME handlers already registered.
    pub(crate) fn prepend_ime<F, R>(&mut self, f: F)
    where
        F: Fn(kivact_types::ImeEvent) -> R + 'static,
        R: KeyHandlerResult,
    {
        self.handlers.ime.caller(move |e| f(e).into_handled());
    }

    /// `on_press`: the shared pointer, Enter/Space and accessibility
    /// Activate action, replacing the caller's.
    pub(crate) fn set_press<F, R>(&mut self, f: F)
    where
        F: Fn() -> R + 'static,
        R: HandlerResult,
    {
        self.handlers.activate = Some(Rc::new(move |_region| f().into_result()));
    }

    /// `on_a11y`: offer an action. `Activate` is reported and ignored (it
    /// is `on_press`'s).
    #[track_caller]
    pub(crate) fn offer_a11y<F, R>(&mut self, action: A11yAction, f: F)
    where
        F: Fn() -> R + 'static,
        R: HandlerResult,
    {
        if action == A11yAction::Activate {
            let at = std::panic::Location::caller();
            let owner = kivact_reactive::Owner::ambient("on_a11y", at);
            let rt = owner.runtime();
            rt.report(
                Some(owner),
                kivact_reactive::UiError::new(
                    kivact_reactive::ErrorKind::Unsupported,
                    "on_a11y(Activate, ..) is ignored; use on_press(..)",
                    Some(at),
                ),
            );
        } else {
            self.offer_a11y_event(action, move |_| f());
        }
    }

    /// `on_a11y_event`: offer an action with the region and value of the
    /// request. A node has one handler per action; `Activate` shares the
    /// activation slot.
    pub(crate) fn offer_a11y_event<F, R>(&mut self, action: A11yAction, f: F)
    where
        F: Fn(A11yEvent) -> R + 'static,
        R: HandlerResult,
    {
        if action == A11yAction::Activate {
            self.handlers.activate = Some(Rc::new(move |region| {
                f(A11yEvent {
                    request: A11yRequest::Activate,
                    region,
                })
                .into_result()
            }));
        } else {
            let h: Rc<dyn Fn(A11yEvent) -> crate::Result<()>> =
                Rc::new(move |event| f(event).into_result());
            let a11y = &mut self.handlers.a11y;
            match a11y.iter_mut().find(|(a, _)| *a == action) {
                Some(entry) => entry.1 = h,
                None => a11y.push((action, h)),
            }
        }
    }
}
