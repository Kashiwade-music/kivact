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
    pub(super) shown: kivact_types::Cursor,
}

impl Default for Pointer {
    fn default() -> Self {
        Self {
            state: PointerState::default(),
            file_target: None,
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
    /// The mouse's window position while it is in the window (it moved
    /// there and has not left, a held button or not), reported with key
    /// events (`HostEvent::Key::pointer`). A touch or pen contact does not
    /// move it.
    pub(crate) mouse: Option<Point>,
}

impl Default for PointerState {
    fn default() -> Self {
        Self {
            tracks: BTreeMap::from([(PointerId::MOUSE.0, Track::new(PointerSource::MOUSE))]),
            mouse: None,
        }
    }
}

/// What the state machine follows of one pointer.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Track {
    /// Which pointer it is; `source.buttons` is the one record of the
    /// buttons it holds.
    pub(crate) source: PointerSource,
    /// The window position of its last event.
    pub(crate) at: Point,
    /// The hit node and its ancestors; always empty for a pointer that is
    /// not primary (it hovers nothing).
    pub(crate) hover: Vec<NodeId>,
    /// The nearest pointer listener under the pointer (`Enter`/`Leave`).
    pub(crate) over: Option<NodeId>,
    /// What its buttons do now.
    pub(crate) gesture: Gesture,
}

/// What a pointer's buttons do, one mode at a time.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Gesture {
    /// Its events go to the listeners (`Route::default()` between gestures).
    Route(Route),
    /// The mouse's primary press landed on a scrollbar: its moves drag the bar
    /// and its other buttons do nothing until the primary release. Only the
    /// mouse's track.
    Scrollbar(ScrollDrag),
    /// A single touch moved past the slop over a scroll container: its capture
    /// and press were cancelled and its moves scroll natively until it ends.
    /// Only a touch track.
    Pan(TouchPan),
}

impl Default for Gesture {
    fn default() -> Self {
        Gesture::Route(Route::default())
    }
}

/// A routed gesture: each part lives while its condition holds.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Route {
    /// The listener that received the gesture's first `Down`; it gets every
    /// later `Down`, `Move`, `Up` and `Cancel` until the last button releases.
    pub(crate) capture: Option<NodeId>,
    /// While a primary pointer holds its primary button.
    pub(crate) press: Option<Press>,
    /// A single touch over a scroll container, still within the slop.
    pub(super) touch: Option<TouchCandidate>,
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
        Self {
            source,
            at: Point::default(),
            hover: Vec::new(),
            over: None,
            gesture: Gesture::default(),
        }
    }

    /// The listener holding the capture, if any.
    pub(crate) fn capture(&self) -> Option<NodeId> {
        match &self.gesture {
            Gesture::Route(route) => route.capture,
            _ => None,
        }
    }

    /// The pressed chain (empty without a press).
    pub(crate) fn pressed(&self) -> &[NodeId] {
        match &self.gesture {
            Gesture::Route(Route {
                press: Some(press), ..
            }) => &press.chain,
            _ => &[],
        }
    }

    /// The routed gesture; only called where the pointer is routed.
    pub(super) fn route_mut(&mut self) -> &mut Route {
        let Gesture::Route(route) = &mut self.gesture else {
            unreachable!(
                "invariant: a press is routed only while the pointer drags no scrollbar and pans nothing"
            )
        };
        route
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
    /// The pressure at its last event, when reported.
    pub(crate) pressure: Option<f64>,
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
            track.hover.retain(|n| tree.live(*n));
            if track.over.is_some_and(|n| !tree.live(n)) {
                track.over = None;
            }
            let ended = match &mut track.gesture {
                Gesture::Route(route) => {
                    if route.capture.is_some_and(|n| !tree.live(n)) {
                        route.capture = None;
                    }
                    if let Some(press) = &mut route.press {
                        press.chain.retain(|n| tree.live(*n));
                        if press.target.is_some_and(|t| !tree.live(t.node)) {
                            press.target = None;
                        }
                    }
                    false
                }
                Gesture::Scrollbar(drag) => !tree.live(drag.node),
                Gesture::Pan(_) => false,
            };
            if ended {
                track.gesture = Gesture::default();
            }
        }
    }
}
