//! Role: The pointer group's state: the machine's tracks and the shell's input.

use std::collections::BTreeMap;

use crate::tree::NodeTree;
use kivact_types::PointerId;
use kivact_types::{FocusTarget, NodeId, Point, PointerKind};

/// The pointer group: the state machine's tracks (with each pointer's last
/// position and its gesture), the file drop target and the cursor last
/// shown. A track keeps only what the machine decided about its pointer;
/// the buttons, pressure, tilt and time of each event come with its sample.
pub(crate) struct Pointer {
    pub(super) state: PointerState,
    pub(super) file_target: Option<NodeId>,
    pub(super) hover_sent: kivact_types::cell::Sent<Option<FocusTarget>>,
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
    pub(crate) id: PointerId,
    pub(crate) over: Option<NodeId>,
    pub(crate) at: Point,
    pub(crate) role: ContactRole,
    pub(crate) gesture: TouchGesture,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PenTrack {
    pub(crate) id: PointerId,
    pub(crate) over: Option<NodeId>,
    pub(crate) at: Point,
    pub(crate) role: ContactRole,
    pub(crate) gesture: PenGesture,
}

/// What a drag that selects text extends by.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SelectUnit {
    /// Characters: from the press to the pointer.
    Char,
    /// Words: the double-clicked word and the words up to the pointer.
    Word(std::ops::Range<usize>),
    /// Lines: the triple-clicked line and the lines up to the pointer.
    Line(std::ops::Range<usize>),
}

/// What a pen contact is doing.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PenGesture {
    Route(Route),
    SelectText {
        editor: NodeId,
        route: Route,
        unit: SelectUnit,
    },
}

impl Default for PenGesture {
    fn default() -> Self {
        Self::Route(Route::default())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MouseGesture {
    Route(Route),
    Scrollbar(ScrollDrag),
    SelectText {
        editor: NodeId,
        route: Route,
        unit: SelectUnit,
    },
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
    SelectText {
        editor: NodeId,
        route: Route,
        unit: SelectUnit,
    },
}

impl Default for TouchGesture {
    fn default() -> Self {
        Self::Route(Route::default())
    }
}

/// Capture and primary press are independent facts of a routed gesture.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Route {
    pub(crate) capture: Option<Capture>,
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
            over: None,
            at: Point::default(),
            hover: Hover::default(),
            gesture: MouseGesture::default(),
        })
    }

    /// The track of a touch or pen contact of `kind` that just went down;
    /// `primary` is the platform's `isPrimary` at its `Down`.
    pub(crate) fn contact(id: PointerId, kind: PointerKind, primary: bool) -> Self {
        debug_assert_ne!(kind, PointerKind::Mouse, "the mouse is not a contact");
        let role = ContactRole::of(primary);
        if kind == PointerKind::Pen {
            Self::Pen(PenTrack {
                id,
                over: None,
                at: Point::default(),
                role,
                gesture: PenGesture::default(),
            })
        } else {
            Self::Touch(TouchTrack {
                id,
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

    /// The pointer's id (the mouse is `PointerId::MOUSE`).
    pub(crate) fn id(&self) -> PointerId {
        match self {
            Self::Mouse(_) => PointerId::MOUSE,
            Self::Touch(t) => t.id,
            Self::Pen(t) => t.id,
        }
    }

    /// Mouse, touch or pen.
    pub(crate) fn kind(&self) -> PointerKind {
        match self {
            Self::Mouse(_) => PointerKind::Mouse,
            Self::Touch(_) => PointerKind::Touch,
            Self::Pen(_) => PointerKind::Pen,
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
                gesture:
                    TouchGesture::Route(r)
                    | TouchGesture::Pending { route: r, .. }
                    | TouchGesture::SelectText { route: r, .. },
                ..
            }) => Some(r),
            Self::Pen(PenTrack {
                gesture: PenGesture::Route(r) | PenGesture::SelectText { route: r, .. },
                ..
            }) => Some(r),
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
                gesture:
                    TouchGesture::Route(r)
                    | TouchGesture::Pending { route: r, .. }
                    | TouchGesture::SelectText { route: r, .. },
                ..
            }) => Some(r),
            Self::Pen(PenTrack {
                gesture: PenGesture::Route(r) | PenGesture::SelectText { route: r, .. },
                ..
            }) => Some(r),
            _ => None,
        }
    }

    /// The editor and unit a drag of this pointer selects text in, if it does.
    pub(crate) fn selecting(&self) -> Option<(NodeId, &SelectUnit)> {
        match self {
            Self::Mouse(t) => match &t.gesture {
                MouseGesture::SelectText { editor, unit, .. } => Some((*editor, unit)),
                _ => None,
            },
            Self::Touch(t) => match &t.gesture {
                TouchGesture::SelectText { editor, unit, .. } => Some((*editor, unit)),
                _ => None,
            },
            Self::Pen(t) => match &t.gesture {
                PenGesture::SelectText { editor, unit, .. } => Some((*editor, unit)),
                PenGesture::Route(_) => None,
            },
        }
    }

    /// Select text in `editor` by `unit` while this pointer is pressed (its route kept).
    pub(crate) fn start_selecting(&mut self, editor: NodeId, unit: SelectUnit) {
        match self {
            Self::Mouse(t) => {
                if let MouseGesture::Route(route) = &mut t.gesture {
                    t.gesture = MouseGesture::SelectText {
                        editor,
                        route: std::mem::take(route),
                        unit,
                    };
                }
            }
            Self::Touch(t) => {
                if let TouchGesture::Route(route) = &mut t.gesture {
                    t.gesture = TouchGesture::SelectText {
                        editor,
                        route: std::mem::take(route),
                        unit,
                    };
                }
            }
            Self::Pen(t) => {
                if let PenGesture::Route(route) = &mut t.gesture {
                    t.gesture = PenGesture::SelectText {
                        editor,
                        route: std::mem::take(route),
                        unit,
                    };
                }
            }
        }
    }

    /// The press ended: a text selection goes back to plain routing.
    pub(crate) fn stop_selecting(&mut self) {
        match self {
            Self::Mouse(t) => {
                if let MouseGesture::SelectText { route, .. } = &mut t.gesture {
                    t.gesture = MouseGesture::Route(std::mem::take(route));
                }
            }
            Self::Touch(t) => {
                if let TouchGesture::SelectText { route, .. } = &mut t.gesture {
                    t.gesture = TouchGesture::Route(std::mem::take(route));
                }
            }
            Self::Pen(t) => {
                if let PenGesture::SelectText { route, .. } = &mut t.gesture {
                    t.gesture = PenGesture::Route(std::mem::take(route));
                }
            }
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
            Self::Pen(t) => t.gesture = PenGesture::default(),
        }
        capture
    }
}

pub(crate) type CursorKind = kivact_types::Cursor;

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
            if track
                .selecting()
                .is_some_and(|(editor, _)| !tree.live(editor))
            {
                track.stop_selecting();
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
