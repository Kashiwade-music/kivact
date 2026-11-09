//! Role: Overlay stacking and popup management.
//!
//! Overlays and modals on the core side: the overlay builder, the modal
//! focus restore and the routing of keys the client applies while overlays
//! are open. The engine is the one authority for modals (`kivact-engine`,
//! `model/modal.rs`): it confines input, Tab order and focus, dismisses on
//! an outside press, and answers through its `Reader` which node an event
//! may reach (`admit_input`), the topmost shown modal (`active_modal`),
//! the overlay an Escape dismisses (`dismiss_target`) and whether a focus
//! restore may land (`can_take_focus`). Core keeps no index of overlays
//! and no admission state; its one modal job is queueing the restore
//! request when a modal overlay unmounts.
//!
//! Invariants:
//! - Admission is the engine's, asked once per event right before the event
//!   is dispatched (`crate::client::dispatch`): while a modal overlay is
//!   shown, `Activate`, `A11y`, `Wheel`, `File`, `TextChanged` and pointer
//!   presses, entries and hovers aimed at a node outside the topmost shown
//!   modal are dropped. The end of a drag that began before the modal
//!   opened (held moves, `Up`, `Cancel` to the node that got its `Down`)
//!   and `Leave` still are admitted; the end of any other drag outside the
//!   modal is not.
//! - Pointer, wheel and key handler chains that start inside the topmost
//!   shown modal stop at it (`ViewNodes::modal_boundary`, [`KeyRoute::stop`]): no
//!   handler behind the modal runs for input inside it.
//! - A key goes to the node the host names, else the focused node, else
//!   the topmost open modal, else the App's view (the first host node the
//!   App mounted below the root), else the root; a target outside the
//!   modal is replaced by the modal ([`key_route`]), and the key bubbles to
//!   the ancestors from there. Starting at the App's view when neither a
//!   focus nor a modal exists is the contract that makes a top-level
//!   `on_key` a window-wide shortcut. A key that reached the App's view
//!   only because nothing has the focus does not activate it (no Enter or
//!   Space activation, [`KeyRoute::activates`]).
//! - An Escape (no modifiers, not a repeat) that no key handler returned
//!   `Handled::Yes` for dismisses the topmost shown overlay offering
//!   `Dismiss`, inside the modal when one is shown ([`escape_dismisses`],
//!   `Reader::dismiss_target`).
//! - A modal overlay gives the focus back to the node that had it when the
//!   overlay mounted, once it unmounts: the restore is a cleanup of the
//!   owner it was mounted under, like every node's unmount. A focus request
//!   of the App wins, and a node that went away, is hidden, disabled or
//!   outside a modal that stays open does not get it back (the one restore
//!   rule of `crate::view::requests`, the engine's `Reader::can_take_focus`).

use std::panic::Location;
use std::rc::Rc;

use kivact_engine::api::PropValue;
use kivact_engine::contract::NodeType;
use kivact_runtime::EffectKind;
use kivact_runtime::HandlerResult;
use kivact_runtime::{ErrorKind, UiError};
use kivact_runtime::{Owner, Runtime};
use kivact_types::{A11yRequest, NodeId, OverlayProps};

use kivact_runtime::{IntoValue, Prop as InputProp};

use crate::view::descriptor::{IntoView, View, mount, view_fn};
use crate::view::element_data::{Content, Element, Node};
use crate::view::input::{DismissReason, Key, KeyEvent, Modifiers};
use crate::view::mount::mount_host;
use crate::view::node_ref::{self, NodeRef};
use crate::view::state::ViewState;
use crate::view::{A11yAction, focus, requests};
use kivact_style::draw::Point;

/// Placement of an overlay relative to its anchor. When the overlay does
/// not fit on that side, the host flips it to the opposite one (`Below` ↔
/// `Above`, `Start` ↔ `End`); it is then kept inside the window.
///
/// `Below`: Below the anchor.
///
/// `Above`: Above the anchor.
///
/// `Start`: Before the anchor in the reading direction.
///
/// `End`: After the anchor in the reading direction.
///
/// `Center`: Centered on the anchor.
#[doc(inline)]
pub use kivact_types::Placement;

/// Alignment of an overlay along its anchor's edge: where the overlay sits
/// on the side [`Placement`] chose. It is not the flex `Align` of a
/// container's children.
///
/// With `Placement::Below`, `Start` lines the overlay's start edge up with
/// the anchor's, `Center` centres it on the anchor and `End` lines its end
/// edge up with the anchor's. A start or end that would overflow the window
/// flips to the other one when that fits.
///
/// `Start`: At the anchor's start edge.
///
/// `Center`: Centred on the anchor.
///
/// `End`: At the anchor's end edge.
#[doc(inline)]
pub use kivact_types::OverlayAlign;

/// Where an overlay is placed, chosen by its constructor: the reactive source
/// of [`kivact_types::OverlayAt`] (the same two cases).
enum Place {
    /// [`overlay`]: against the node `node` is bound to; `at_least_width`
    /// from [`Overlay::at_least_anchor_width`].
    Anchor { node: NodeRef, at_least_width: bool },
    /// [`overlay_at`]: at a window point, constant or reactive.
    /// `width_misuse`: where `.at_least_anchor_width()` was called on it,
    /// reported once at mount and ignored (a point has no width).
    Point {
        point: InputProp<Point>,
        width_misuse: Option<&'static Location<'static>>,
    },
}

/// Out-of-flow layer (popup, menu, tooltip). Built by [`overlay`] and
/// [`overlay_at`].
pub struct Overlay {
    place: Place,
    placement: InputProp<Placement>,
    align: InputProp<OverlayAlign>,
    offset: InputProp<f64>,
    viewport_margin: Option<InputProp<f64>>,
    keep_anchor_clear: bool,
    modal: bool,
    child: View,
    /// Holds the modifiers (`Element`); its content and children are unused.
    root: Node,
    at: &'static Location<'static>,
}

/// Show `child` in an out-of-flow layer placed relative to the node bound to
/// `anchor` (a popup, menu or tooltip). The host flips it to the other side
/// of the anchor when it does not fit and keeps it inside the window; with
/// an unbound anchor it is centered in the window. `.align(..)` lines it up
/// with the anchor's start, centre or end, `.offset(px)` leaves a gap,
/// `.modal()` blocks input to the rest of the window while it is
/// shown, and `.on_dismiss(f)` closes it on an outside press or Escape.
///
/// ```
/// # use kivact_core as kivact;
/// use kivact::state::{signal};
/// use kivact::view::{IntoView, Element, Role, node, node_ref, show, text};
/// use kivact::view::{OverlayAlign, Placement, overlay};
/// fn menu_button() -> impl IntoView {
///     let open = signal(false);
///     let button = node_ref();
///     (
///         node()
///             .node_ref(button)
///             .role(Role::Button)
///             .on_press(move || open.set(true)),
///         // .menu--right { top: calc(100% + 6px); right: 0 }
///         show(open, move || {
///             overlay(button, Placement::Below, text("Menu"))
///                 .align(OverlayAlign::End)
///                 .offset(6)
///                 .on_dismiss(move |_| open.set(false))
///         }),
///     )
/// }
/// ```
///
/// An overlay is the floating layer of a component (menus, popovers,
/// tooltips, dialogs): placed at an anchor, flipped and kept in the
/// window, with Dismiss and modal semantics. A box that stays put in the
/// window (a toast stack, an off-canvas sidebar and its scrim) is
/// `style().position(Position::Fixed)` with insets instead.
#[track_caller]
pub fn overlay(
    anchor: NodeRef,
    placement: impl IntoValue<Placement>,
    child: impl IntoView,
) -> Overlay {
    Overlay {
        place: Place::Anchor {
            node: anchor,
            at_least_width: false,
        },
        placement: placement.into_prop(),
        align: InputProp::constant(OverlayAlign::Start),
        offset: InputProp::constant(0.0),
        viewport_margin: None,
        keep_anchor_clear: false,
        modal: false,
        child: child.into_view(),
        root: Node::new(Content::Box, None),
        at: Location::caller(),
    }
}

/// Show `child` in an out-of-flow layer placed at a window point (a
/// context menu at the pointer: `PointerEvent::window_pos`; a pie menu
/// centered on it with `Placement::Center`), constant or reactive. It
/// flips and stays inside the window like [`overlay`]; the press that
/// dismisses it is any press outside its subtree.
///
/// ```
/// # use kivact_core as kivact;
/// use kivact::state::{signal};
/// use kivact::view::{IntoView, MouseButton, Node, Element, show_some};
/// use kivact::geometry::{Point};
/// use kivact::view::{Placement, PointerEvent};
/// use kivact::view::{View, overlay_at};
/// fn with_context_menu(area: Node, menu: impl Fn() -> View + 'static) -> impl IntoView {
///     let at = signal(None::<Point>);
///     (
///         area.on_pointer(move |e: PointerEvent| {
///             if let kivact::view::PointerPhase::Down(press) = e.phase()
///                 && press.button == kivact::view::MouseButton::Secondary
///             {
///                 at.set(Some(e.window_pos()));
///             }
///         }),
///         show_some(at, move |p| {
///             overlay_at(p, Placement::Below, menu()).on_dismiss(move |_| at.set(None))
///         }),
///     )
/// }
/// ```
#[track_caller]
pub fn overlay_at(
    point: impl IntoValue<Point>,
    placement: impl IntoValue<Placement>,
    child: impl IntoView,
) -> Overlay {
    Overlay {
        place: Place::Point {
            point: point.into_prop(),
            width_misuse: None,
        },
        placement: placement.into_prop(),
        align: InputProp::constant(OverlayAlign::Start),
        offset: InputProp::constant(0.0),
        viewport_margin: None,
        keep_anchor_clear: false,
        modal: false,
        child: child.into_view(),
        root: Node::new(Content::Box, None),
        at: Location::caller(),
    }
}

impl Overlay {
    /// Where along the anchor's side it sits: its start (the default),
    /// centre or end. `overlay(menu_button, Placement::Below, ..)
    /// .align(OverlayAlign::End)` lines its right edge up with the button's
    /// (CSS `right: 0`); a submenu `overlay(item, Placement::End, ..)` sits
    /// top-aligned with its item by default. A start or end that would
    /// overflow the window flips to the other one when that fits.
    pub fn align(mut self, align: impl IntoValue<OverlayAlign>) -> Self {
        self.align = align.into_prop();
        self
    }

    /// The gap between the anchor and the overlay in logical px (CSS
    /// `top: calc(100% + 6px)` is `.offset(6)` below the anchor); it
    /// follows the overlay when it flips to the other side.
    pub fn offset(mut self, px: impl IntoValue<f64>) -> Self {
        self.offset = px.into_prop();
        self
    }

    /// Inset the available window area by this many logical pixels.
    /// Overrides the theme's [`kivact_style::Theme::overlay_margin`].
    pub fn viewport_margin(mut self, px: impl IntoValue<f64>) -> Self {
        self.viewport_margin = Some(px.into_prop());
        self
    }

    /// Prefer placements that leave the anchor clear when viewport fitting is active.
    pub fn keep_anchor_clear(mut self) -> Self {
        self.keep_anchor_clear = true;
        self
    }

    /// Block input to the rest of the window while the overlay is shown.
    pub fn modal(mut self) -> Self {
        self.modal = true;
        self
    }

    /// Make the overlay at least as wide as its anchor (CSS
    /// `min-width: var(--anchor-width)`): the host raises its minimum width
    /// to the anchor's layout width before laying it out, in the same frame,
    /// and follows the anchor when its width changes. A wider content
    /// keeps its width; a percentage or `auto` minimum width of the overlay
    /// gives way to the anchor's. On an overlay placed at a point
    /// ([`overlay_at`]) it is reported once at mount
    /// (`ErrorKind::Unsupported`) and ignored; with an unbound anchor it does
    /// nothing. A select's list under its trigger uses it.
    #[track_caller]
    pub fn at_least_anchor_width(mut self) -> Self {
        match &mut self.place {
            Place::Anchor { at_least_width, .. } => *at_least_width = true,
            Place::Point { width_misuse, .. } => *width_misuse = Some(Location::caller()),
        }
        self
    }

    /// Ask to close: `f` runs with the [`DismissReason`] when a pointer
    /// press lands outside the overlay and outside its anchor (the host),
    /// when Escape reaches no key handler that returns `Handled::Yes` and
    /// this is the topmost overlay offering it (the core), and on an
    /// assistive technology's Dismiss action (hosts expose only nodes with
    /// a role to assistive technology: give the overlay one, such as
    /// `Role::Menu` or `Role::Dialog`, for its Dismiss to reach it).
    /// Closing is up to `f` (usually it sets the signal that shows the
    /// overlay; ignore the reason with `|_|`). A transactional editor can
    /// commit on an outside press and restore its original value on Escape.
    /// Registering it again replaces the earlier handler, as for
    /// `.a11y_action(A11yAction::Dismiss, ..)`.
    pub fn on_dismiss<F, R>(mut self, f: F) -> Self
    where
        F: Fn(DismissReason) -> R + 'static,
        R: HandlerResult,
    {
        self.root
            .common
            .offer_a11y_event(A11yAction::Dismiss, move |event| f(dismiss_reason(&event)));
        self
    }

    /// Append a popup controller's dismissal behavior after an existing
    /// callback. An error prevents the appended behavior. Ordinary dismissal
    /// registration still replaces this single accessibility action.
    pub fn on_dismiss_default<F, R>(mut self, f: F) -> Self
    where
        F: Fn(DismissReason) -> R + 'static,
        R: HandlerResult,
    {
        let action = A11yAction::Dismiss;
        let entries = &mut self.root.common.handlers.a11y;
        let previous = entries
            .iter()
            .find(|(a, _)| *a == action)
            .map(|(_, h)| h.clone());
        let handler = kivact_runtime::Handler::from_fn(move |event| {
            let reason = dismiss_reason(&event);
            if let Some(previous) = &previous {
                previous.call(event)?;
            }
            f(reason).into_outcome().map(|_| ())
        });
        match entries.iter_mut().find(|(a, _)| *a == action) {
            Some(entry) => entry.1 = handler,
            None => entries.push((action, handler)),
        }
        self
    }
}

/// The reason of a dismissal request: only `Dismiss` requests reach the
/// Dismiss handler slot.
fn dismiss_reason(event: &crate::view::A11yEvent) -> DismissReason {
    match event.request {
        A11yRequest::Dismiss(reason) => reason,
        _ => unreachable!("invariant: the Dismiss slot receives only Dismiss requests"),
    }
}

impl Element for Overlay {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Overlay {
    fn into_view(self) -> View {
        view_fn(move |cx| {
            let Overlay {
                place,
                placement,
                align,
                offset,
                viewport_margin,
                keep_anchor_clear,
                modal,
                child,
                root,
                at,
            } = self;
            let mut common = root.into_modifiers(cx, "an overlay", at);
            let part_modal = std::mem::take(&mut common.part_modal);
            let modal = modal || part_modal;
            if let Place::Point {
                width_misuse: Some(loc),
                ..
            } = &place
            {
                cx.rt.report(
                    Some(cx.owner),
                    UiError::new(
                        ErrorKind::Unsupported,
                        "`.at_least_anchor_width()` on an overlay placed at a point (`overlay_at`): a point has no width; ignored",
                        Some(loc),
                    ),
                );
            }
            let id = mount_host(cx, NodeType::Overlay, common, at);
            let weak = cx.weak();
            cx.effect(EffectKind::Render, at, move || {
                let at_place = match &place {
                    Place::Anchor {
                        node,
                        at_least_width,
                    } => kivact_types::OverlayAt::Anchor {
                        node: node_ref::node_value(node, at),
                        at_least_width: *at_least_width,
                    },
                    Place::Point { point, .. } => {
                        kivact_types::OverlayAt::Point(kivact_runtime::Prop::get(point))
                    }
                };
                if let Some(rt) = weak.upgrade() {
                    let props = OverlayProps {
                        at: at_place,
                        placement: placement.get(),
                        align: align.get(),
                        offset: offset.get(),
                        fit: viewport_margin
                            .as_ref()
                            .map(InputProp::get)
                            .or_else(|| Some(kivact_style::use_theme().get().overlay_margin()))
                            .map(|margin| kivact_types::OverlayFit {
                                margin,
                                keep_anchor_clear,
                                rtl: crate::view::use_direction().get()
                                    == crate::view::ReadingDirection::Rtl,
                            }),
                        modal,
                    };
                    ViewState::tree_mut(&rt).update(
                        id,
                        PropValue::Overlay(props),
                        rt.running_effect(),
                    );
                }
            });
            let c = mount(child, cx);
            cx.append(id, c);
            if modal {
                restore_focus_on_unmount(cx.rt, cx.window, cx.owner, id);
            }
            id
        })
    }
}

/// When the modal overlay `id` (mounted under `owner`) unmounts, give the
/// focus back to the node that had it now, if that node is still live and
/// the focus is inside the removed overlay or nowhere. A node removed,
/// hidden, disabled or made unfocusable in the meantime, or outside the
/// modal that stays open, gets no request and no error, and a
/// `NodeRef::focus` the App asks for in the same turn wins
/// (`requests::flush_focus`, `Reader::can_take_focus`).
fn restore_focus_on_unmount(
    rt: &Rc<Runtime>,
    window: kivact_types::WindowId,
    owner: Owner,
    id: NodeId,
) {
    let Some(opener) = focus::focused(rt, window) else {
        return;
    };
    let visible = ViewState::of(rt)
        .focus_keys(window)
        .is_some_and(|keys| keys.visible.get_untracked());
    let scope = ModalScope {
        rt: Rc::downgrade(rt),
        window,
        opener,
        id,
        visible,
    };
    owner.store("ModalScope", Rc::new(scope), Location::caller());
}
struct ModalScope {
    rt: std::rc::Weak<Runtime>,
    window: kivact_types::WindowId,
    opener: kivact_types::FocusTarget,
    id: NodeId,
    visible: bool,
}
impl Drop for ModalScope {
    fn drop(&mut self) {
        let Self {
            rt: weak,
            window,
            opener,
            id,
            visible,
        } = self;
        let (window, opener, id, visible) = (*window, *opener, *id, *visible);
        let Some(rt) = weak.upgrade() else { return };
        let back = {
            let tree = ViewState::tree(&rt);
            let inside = |target: kivact_types::FocusTarget| {
                let f = target.node;
                // Already deleted with the overlay, or still under it.
                !tree.contains(f)
                    || std::iter::successors(Some(f), |n| tree.reader().parent(*n)).any(|n| n == id)
            };
            tree.contains(opener.node) && focus::focused(&rt, window).is_none_or(inside)
        };
        if back {
            requests::request_restore_focus(&rt, opener, None, visible);
        }
    }
}

/// Where a key goes and where its handler chain stops (see [`key_route`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KeyRoute {
    /// The first node of the chain.
    pub target: NodeId,
    // xtask-allow(state-fields): per-dispatch result value - The topmost shown modal, where the chain stops
    /// The topmost shown modal, where the chain stops.
    pub stop: Option<NodeId>,
    // xtask-allow(state-fields): per-dispatch result value - Whether an unhandled Enter or Space activates `target`: not when the key reached the App's view (or the root) only because nothing has the focus and no modal is open
    /// Whether an unhandled Enter or Space activates `target`: not when
    /// the key reached the App's view (or the root) only because nothing
    /// has the focus and no modal is open.
    pub activates: bool,
}

/// The route of a key the host aimed at `named` in `window`: that node, else
/// the focused node, else the topmost open modal, else the App's view (so
/// window-level shortcuts on it see keys while nothing has the focus),
/// else the root; a target outside the open modal is replaced by the
/// modal, and the chain stops at the modal.
pub(crate) fn key_route(
    rt: &Runtime,
    window: kivact_types::WindowId,
    named: Option<NodeId>,
) -> KeyRoute {
    let focused = focus::focused(rt, window).map(|f| f.node);
    let tree = ViewState::tree(rt);
    let modal = tree.reader().active_modal(window);
    let target = named
        .or(focused)
        .filter(|t| modal.is_none_or(|m| tree.is_within(*t, m)))
        .or(modal);
    let root = tree.root(window).unwrap_or(NodeId(1));
    KeyRoute {
        activates: target.is_some(),
        target: target
            .or_else(|| tree.reader().children(root).first().copied())
            .unwrap_or(root),
        stop: modal,
    }
}

/// The overlay a key in `window` no handler returned `Handled::Yes` for
/// dismisses: an Escape without modifiers that is not a repeat goes to the
/// topmost shown overlay offering `Dismiss` (inside the modal when one is
/// shown), as the engine decides.
pub(crate) fn escape_dismisses(
    rt: &Runtime,
    window: kivact_types::WindowId,
    e: &KeyEvent,
) -> Option<NodeId> {
    if e.repeat() || e.key() != Key::Escape || e.modifiers() != Modifiers::default() {
        return None;
    }
    ViewState::tree(rt).reader().dismiss_target(window)
}
