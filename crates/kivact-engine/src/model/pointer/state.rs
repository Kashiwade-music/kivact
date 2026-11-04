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
    pub(super) file_target: Option<NodeId>,
    // xtask-allow(state-fields): Sent is the last hover report, independent of file drag state.
    pub(super) hover_sent: kivact_types::cell::Sent<Option<FocusTarget>>,
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
            tracks: BTreeMap::from([(PointerId::MOUSE.0, Track::new(PointerSource::MOUSE))]),
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
    // xtask-allow(state-fields): entered listener and primary hover are independent input facts.
    pub(crate) over: Option<NodeId>,
    pub(crate) at: Option<Point>,
    pub(crate) hover: Hover,
    pub(crate) gesture: MouseGesture,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TouchTrack {
    pub(crate) source: PointerSource,
    // xtask-allow(state-fields): entered listener and primary hover are independent input facts.
    pub(crate) over: Option<NodeId>,
    pub(crate) at: Point,
    pub(crate) hover: Option<Hover>,
    pub(crate) gesture: TouchGesture,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PenTrack {
    pub(crate) source: PointerSource,
    // xtask-allow(state-fields): entered listener and primary hover are independent input facts.
    pub(crate) over: Option<NodeId>,
    pub(crate) at: Point,
    pub(crate) hover: Option<Hover>,
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
    // xtask-allow(state-fields): capture and primary press have independent lifetimes.
    pub(crate) capture: Option<NodeId>,
    pub(crate) press: Option<Press>,
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
    pub(super) start: Point,
    pub(super) target: TouchTarget,
}

/// A touch that scrolls natively along `axis`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TouchPan {
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
    pub(crate) fn new(source: PointerSource) -> Self {
        match source.kind {
            PointerKind::Mouse => Self::Mouse(MouseTrack {
                source,
                over: None,
                at: None,
                hover: Hover::default(),
                gesture: MouseGesture::default(),
            }),
            PointerKind::Touch => Self::Touch(TouchTrack {
                source,
                over: None,
                at: Point::default(),
                hover: source.primary.then(Hover::default),
                gesture: TouchGesture::default(),
            }),
            PointerKind::Pen => Self::Pen(PenTrack {
                source,
                over: None,
                at: Point::default(),
                hover: source.primary.then(Hover::default),
                route: Route::default(),
            }),
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
            Self::Mouse(t) => t.at.unwrap_or_default(),
            Self::Touch(t) => t.at,
            Self::Pen(t) => t.at,
        }
    }

    pub(super) fn set_at(&mut self, at: Point) {
        match self {
            Self::Mouse(t) => t.at = Some(at),
            Self::Touch(t) => t.at = at,
            Self::Pen(t) => t.at = at,
        }
    }

    pub(crate) fn hover(&self) -> Option<&Hover> {
        match self {
            Self::Mouse(t) => Some(&t.hover),
            Self::Touch(t) => t.hover.as_ref(),
            Self::Pen(t) => t.hover.as_ref(),
        }
    }

    pub(super) fn hover_mut(&mut self) -> Option<&mut Hover> {
        match self {
            Self::Mouse(t) => Some(&mut t.hover),
            Self::Touch(t) => t.hover.as_mut(),
            Self::Pen(t) => t.hover.as_mut(),
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
        self.route().and_then(|r| r.capture)
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
    // xtask-allow(state-fields): optional shell sample data, independent of primary identity.
    /// The pressure at its last event, when reported.
    pub(crate) pressure: Option<f64>,
    // xtask-allow(state-fields): optional shell sample data, independent of pressure.
    /// The pen's tilt at its last event, when reported.
    pub(crate) tilt: Option<(f64, f64)>,
    /// The buttons held after its last event (none after a `Cancel`).
    pub(crate) buttons: PointerButtons,
    /// Whether it is the primary pointer of its kind (Pointer Events'
    /// `isPrimary`): the mouse always; a touch or pen contact when no other
    /// contact of its kind was in contact at its `Down`, until it ends.
    pub(crate) primary: bool,
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
        primary: true,
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
                if route.capture.is_some_and(|n| !tree.live(n)) {
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
