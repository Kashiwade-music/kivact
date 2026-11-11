//! Role: The pointer group's state: the machine's tracks and the shell's input.

use std::collections::BTreeMap;

use crate::tree::NodeTree;
use kivact_types::PointerId;
use kivact_types::{FocusTarget, NodeId, Point, PointerButtons, PointerKind};

/// The pointer group: the state machine's tracks (with each pointer's last
/// position and its gesture), the file drop target and the cursor last
/// shown. The buttons a pointer holds are the shell's (`PointerSample::buttons`),
/// kept once in its track's source.
pub(crate) struct Pointer {
    pub(super) state: PointerState,
    // xtask-allow(state-fields): independent: the file-listening node under an OS file drag, set by the file drag events and cleared at Drop, Leave or its removal; the hover and position projections follow mouse events, so any pair occurs
    pub(super) file_target: Option<NodeId>,
    // xtask-allow(state-fields): projection: the hover last reported to the client, written only by Model::report_hover
    pub(super) hover_sent: kivact_types::cell::Sent<Option<FocusTarget>>,
    // xtask-allow(state-fields): projection: the mouse position last reported to the client, written only by Model::report_mouse_position
    pub(super) position_sent: kivact_types::cell::Sent<Option<Point>>,
    pub(super) shown: kivact_types::Cursor,
}

impl Default for Pointer {
    fn default() -> Self {
        Self {
            state: PointerState::default(),
            file_target: None,
            hover_sent: {
                let mut sent = kivact_types::cell::Sent::new();
                sent.update(None);
                sent
            },
            position_sent: {
                let mut sent = kivact_types::cell::Sent::new();
                sent.update(None);
                sent
            },
            shown: kivact_types::Cursor::Default,
        }
    }
}

impl Pointer {
    /// Note `cursor` as shown; whether it differs from the one shown.
    pub(crate) fn show(&mut self, cursor: kivact_types::Cursor) -> bool {
        std::mem::replace(&mut self.shown, cursor) != cursor
    }

    /// Drop removed and leaving nodes: a scrollbar drag of one ends, and
    /// a file drag over one is over nothing.
    pub(crate) fn forget_missing(&mut self, tree: &NodeTree) {
        self.state.forget_missing(tree);
        if self.file_target.is_some_and(|n| !tree.live(n)) {
            self.file_target = None;
        }
    }
}

/// Pointer interaction state: one track per pointer.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PointerState {
    /// Every pointer by its id (`PointerId::0`): the mouse, always, and
    /// each touch or pen contact from its `Down` to its `Up` or `Cancel`.
    /// In id order, so the mouse comes first and the contacts in the order
    /// they began.
    pub(crate) tracks: BTreeMap<u64, Track>,
}

impl Default for PointerState {
    fn default() -> Self {
        Self {
            tracks: BTreeMap::from([(PointerId::MOUSE.0, Track::mouse())]),
        }
    }
}

/// Each pointer kind owns only the gestures it supports.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Track {
    Mouse(MouseTrack),
    Touch(TouchTrack),
    Pen(PenTrack),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Hover {
    pub(crate) chain: Vec<NodeId>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MouseTrack {
    pub(crate) source: PointerSource,
    // xtask-allow(state-fields): independent: the pointer's target and the hover chain are cleared separately
    pub(crate) over: Option<NodeId>,
    pub(crate) at: Point,
    pub(crate) hover: Hover,
    pub(crate) gesture: MouseGesture,
}

/// A contact's part in hover: the primary contact of its kind (Pointer
/// Events' `isPrimary`) keeps a hover chain while it lasts; another
/// contact of the same kind has none.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ContactRole {
    Primary(Hover),
    Secondary,
}

impl ContactRole {
    fn of(primary: bool) -> Self {
        if primary {
            Self::Primary(Hover::default())
        } else {
            Self::Secondary
        }
    }

    fn hover(&self) -> Option<&Hover> {
        match self {
            Self::Primary(hover) => Some(hover),
            Self::Secondary => None,
        }
    }

    fn hover_mut(&mut self) -> Option<&mut Hover> {
        match self {
            Self::Primary(hover) => Some(hover),
            Self::Secondary => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TouchTrack {
    pub(crate) source: PointerSource,
    // xtask-allow(state-fields): independent: the pointer's target and the hover chain are cleared separately
    pub(crate) over: Option<NodeId>,
    pub(crate) at: Point,
    pub(crate) role: ContactRole,
    pub(crate) gesture: TouchGesture,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PenTrack {
    pub(crate) source: PointerSource,
    // xtask-allow(state-fields): independent: the nearest pointer-listening ancestor of the hit, set by track_hover; clear_hover empties the hover chain but keeps it for Leave, and a non-primary pen has no chain, so every pair occurs
    pub(crate) over: Option<NodeId>,
    pub(crate) at: Point,
    pub(crate) role: ContactRole,
    pub(crate) route: Route,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MouseGesture {
    Route(Route),
    Scrollbar(ScrollDrag),
    SelectText { editor: NodeId, route: Route },
}

impl Default for MouseGesture {
    fn default() -> Self {
        Self::Route(Route::default())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TouchGesture {
    Route(Route),
    Pending {
        route: Route,
        candidate: TouchCandidate,
    },
    Pan(TouchPan),
}

impl Default for TouchGesture {
    fn default() -> Self {
        Self::Route(Route::default())
    }
}

/// Capture and primary press are independent facts of a routed gesture.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Route {
    // xtask-allow(state-fields): independent: where the gesture's listener events go: set by any button's Down over a listener or by set_pointer_capture, cleared when no button is held; a secondary-button drag has it without a press
    pub(crate) capture: Option<Capture>,
    // xtask-allow(state-fields): independent: the primary button's pressed chain and activation target, set only by primary_press and taken by that button's Up; a press over a node without a pointer listener has no capture, and a listener may release the capture mid-press
    pub(crate) press: Option<Press>,
}

/// Capture retains both its target and the reason it survives a move.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Capture {
    Implicit(NodeId),
    Explicit(NodeId),
    /// A listener released the capture: until the gesture's final `Up`,
    /// its events are hit-tested, that `Up` included.
    Released,
}

impl Capture {
    /// The node the gesture's events go to, if any is captured.
    pub(crate) fn target(self) -> Option<NodeId> {
        match self {
            Self::Implicit(target) | Self::Explicit(target) => Some(target),
            Self::Released => None,
        }
    }
}

/// A primary press of a primary pointer.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Press {
    /// The hover chain at the press (the pressed chain).
    pub(crate) chain: Vec<NodeId>,
    /// The exact activation target, if any.
    pub(crate) target: Option<FocusTarget>,
}

/// Where a touch's native scroll starts, and what it needs to stay valid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TouchTarget {
    pub(super) from: NodeId,
    pub(super) origin: NodeId,
    pub(super) modal: Option<NodeId>,
}

/// A touch that may start a native scroll once it leaves the slop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TouchCandidate {
    pub(super) time_ns: u64,
    pub(super) start: Point,
    pub(super) target: TouchTarget,
}

/// A touch that scrolls natively along `axis`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TouchPan {
    pub(super) samples: kivact_types::DragVelocity,
    pub(super) scroller: NodeId,
    pub(super) last: Point,
    pub(super) target: TouchTarget,
    pub(super) axis: Axis,
}

/// The axis a touch pan scrolls along.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Axis {
    Horizontal,
    Vertical,
}

impl Track {
    /// The mouse's track (always primary, always hovering).
    pub(crate) fn mouse() -> Self {
        Self::Mouse(MouseTrack {
            source: PointerSource::MOUSE,
            over: None,
            at: Point::default(),
            hover: Hover::default(),
            gesture: MouseGesture::default(),
        })
    }

    /// The track of a touch or pen contact that just went down; `primary`
    /// is the platform's `isPrimary` at its `Down`.
    pub(crate) fn contact(source: PointerSource, primary: bool) -> Self {
        debug_assert_ne!(
            source.kind,
            PointerKind::Mouse,
            "the mouse is not a contact"
        );
        let role = ContactRole::of(primary);
        if source.kind == PointerKind::Pen {
            Self::Pen(PenTrack {
                source,
                over: None,
                at: Point::default(),
                role,
                route: Route::default(),
            })
        } else {
            Self::Touch(TouchTrack {
                source,
                over: None,
                at: Point::default(),
                role,
                gesture: TouchGesture::default(),
            })
        }
    }

    /// Whether this is the primary pointer of its kind (Pointer Events'
    /// `isPrimary`): the mouse always; a contact when it was the only one of
    /// its kind at its `Down`, until it ends.
    pub(crate) fn is_primary(&self) -> bool {
        match self {
            Self::Mouse(_) => true,
            Self::Touch(t) => matches!(t.role, ContactRole::Primary(_)),
            Self::Pen(t) => matches!(t.role, ContactRole::Primary(_)),
        }
    }

    pub(crate) fn source(&self) -> &PointerSource {
        match self {
            Self::Mouse(t) => &t.source,
            Self::Touch(t) => &t.source,
            Self::Pen(t) => &t.source,
        }
    }

    pub(crate) fn source_mut(&mut self) -> &mut PointerSource {
        match self {
            Self::Mouse(t) => &mut t.source,
            Self::Touch(t) => &mut t.source,
            Self::Pen(t) => &mut t.source,
        }
    }

    pub(crate) fn at(&self) -> Point {
        match self {
            Self::Mouse(t) => t.at,
            Self::Touch(t) => t.at,
            Self::Pen(t) => t.at,
        }
    }

    pub(super) fn set_at(&mut self, at: Point) {
        match self {
            Self::Mouse(t) => t.at = at,
            Self::Touch(t) => t.at = at,
            Self::Pen(t) => t.at = at,
        }
    }

    pub(crate) fn hover(&self) -> Option<&Hover> {
        match self {
            Self::Mouse(t) => Some(&t.hover),
            Self::Touch(t) => t.role.hover(),
            Self::Pen(t) => t.role.hover(),
        }
    }

    pub(super) fn hover_mut(&mut self) -> Option<&mut Hover> {
        match self {
            Self::Mouse(t) => Some(&mut t.hover),
            Self::Touch(t) => t.role.hover_mut(),
            Self::Pen(t) => t.role.hover_mut(),
        }
    }

    pub(crate) fn chain(&self) -> &[NodeId] {
        self.hover().map_or(&[], |hover| hover.chain.as_slice())
    }

    pub(super) fn over(&self) -> Option<NodeId> {
        match self {
            Self::Mouse(t) => t.over,
            Self::Touch(t) => t.over,
            Self::Pen(t) => t.over,
        }
    }

    pub(super) fn set_over(&mut self, over: Option<NodeId>) {
        match self {
            Self::Mouse(t) => t.over = over,
            Self::Touch(t) => t.over = over,
            Self::Pen(t) => t.over = over,
        }
    }

    pub(super) fn route(&self) -> Option<&Route> {
        match self {
            Self::Mouse(MouseTrack {
                gesture: MouseGesture::Route(r) | MouseGesture::SelectText { route: r, .. },
                ..
            }) => Some(r),
            Self::Touch(TouchTrack {
                gesture: TouchGesture::Route(r) | TouchGesture::Pending { route: r, .. },
                ..
            }) => Some(r),
            Self::Pen(t) => Some(&t.route),
            _ => None,
        }
    }

    pub(super) fn routed_mut(&mut self) -> Option<&mut Route> {
        match self {
            Self::Mouse(MouseTrack {
                gesture: MouseGesture::Route(r) | MouseGesture::SelectText { route: r, .. },
                ..
            }) => Some(r),
            Self::Touch(TouchTrack {
                gesture: TouchGesture::Route(r) | TouchGesture::Pending { route: r, .. },
                ..
            }) => Some(r),
            Self::Pen(t) => Some(&mut t.route),
            _ => None,
        }
    }

    pub(crate) fn capture(&self) -> Option<NodeId> {
        self.route()
            .and_then(|r| r.capture)
            .and_then(Capture::target)
    }

    pub(crate) fn pressed(&self) -> &[NodeId] {
        self.route()
            .and_then(|r| r.press.as_ref())
            .map_or(&[], |p| p.chain.as_slice())
    }

    pub(super) fn route_mut(&mut self) -> &mut Route {
        self.routed_mut()
            .expect("invariant: the pointer gesture is routed")
    }

    pub(super) fn reset_gesture(&mut self) -> Option<NodeId> {
        let capture = self.capture();
        match self {
            Self::Mouse(t) => t.gesture = MouseGesture::default(),
            Self::Touch(t) => t.gesture = TouchGesture::default(),
            Self::Pen(t) => t.route = Route::default(),
        }
        capture
    }
}

/// One pointer: the mouse, or a touch or pen contact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PointerSource {
    pub(crate) time_ns: u64,
    /// The pointer's id (the mouse is 1).
    pub(crate) id: PointerId,
    /// Mouse, touch or pen.
    pub(crate) kind: PointerKind,
    // xtask-allow(state-fields): independent: the pressure of the pointer's last shell sample (shell.rs); platforms report pressure and tilt apart (a touch may have pressure and no tilt, a mouse neither), so every pair occurs
    /// The pressure at its last event, when reported.
    pub(crate) pressure: Option<f64>,
    // xtask-allow(state-fields): independent: the pen tilt of the pointer's last shell sample (shell.rs); platforms report tilt and pressure apart (a pen may lack either, a mouse has neither), so every pair occurs
    /// The pen's tilt at its last event, when reported.
    pub(crate) tilt: Option<(f64, f64)>,
    /// The buttons held after its last event (none after a `Cancel`).
    pub(crate) buttons: PointerButtons,
}

pub(crate) type CursorKind = kivact_types::Cursor;

impl PointerSource {
    /// The mouse.
    pub(crate) const MOUSE: PointerSource = PointerSource {
        id: PointerId::MOUSE,
        time_ns: 0,
        kind: PointerKind::Mouse,
        pressure: None,
        tilt: None,
        buttons: PointerButtons::NONE,
    };
}

/// A mouse scrollbar drag in progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollDrag {
    pub(crate) node: NodeId,
    pub(crate) vertical: bool,
    pub(crate) pointer: f64,
    pub(crate) offset: f64,
}

impl PointerState {
    /// The track of `id`.
    pub(crate) fn track(&self, id: PointerId) -> &Track {
        self.tracks
            .get(&id.0)
            .expect("invariant: the machine follows the pointer of every event it handles")
    }

    pub(crate) fn track_mut(&mut self, id: PointerId) -> &mut Track {
        self.tracks
            .get_mut(&id.0)
            .expect("invariant: the machine follows the pointer of every event it handles")
    }

    /// Drop removed and leaving nodes.
    pub(crate) fn forget_missing(&mut self, tree: &NodeTree) {
        for track in self.tracks.values_mut() {
            if let Some(hover) = track.hover_mut() {
                hover.chain.retain(|n| tree.live(*n));
            }
            if track.over().is_some_and(|n| !tree.live(n)) {
                track.set_over(None);
            }
            if let Some(route) = track.routed_mut() {
                if route
                    .capture
                    .and_then(Capture::target)
                    .is_some_and(|target| !tree.live(target))
                {
                    route.capture = None;
                }
                if let Some(press) = &mut route.press {
                    press.chain.retain(|n| tree.live(*n));
                    if press.target.is_some_and(|t| !tree.live(t.node)) {
                        press.target = None;
                    }
                }
            }
            if let Track::Mouse(mouse) = track
                && let MouseGesture::SelectText { editor, route } = &mut mouse.gesture
                && !tree.live(*editor)
            {
                mouse.gesture = MouseGesture::Route(std::mem::take(route));
            }
            if matches!(track, Track::Mouse(MouseTrack { gesture: MouseGesture::Scrollbar(drag), .. }) if !tree.live(drag.node))
            {
                track.reset_gesture();
            }
        }
    }

    pub(crate) fn mouse(&self) -> &MouseTrack {
        let Track::Mouse(mouse) = self.track(PointerId::MOUSE) else {
            unreachable!("invariant: the mouse id belongs to the mouse");
        };
        mouse
    }

    pub(super) fn mouse_mut(&mut self) -> &mut MouseTrack {
        let Track::Mouse(mouse) = self.track_mut(PointerId::MOUSE) else {
            unreachable!("invariant: the mouse id belongs to the mouse");
        };
        mouse
    }
}
