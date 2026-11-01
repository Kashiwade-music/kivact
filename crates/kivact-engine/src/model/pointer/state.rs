//! Role: The pointer group's state: the machine's tracks and the shell's input.

use std::collections::BTreeMap;

use crate::tree::NodeTree;
use kivact_wire::PointerId;
use kivact_wire::{FocusTarget, MouseButton, NodeId, Point, PointerButtons, PointerKind};

/// The pointer group: the state machine's tracks (with each pointer's last
/// position), and the shell's pointer input it is driven by (the mouse's
/// held buttons, a scrollbar drag, the file drop target) and the cursor
/// last shown.
pub(crate) struct Pointer {
    pub(super) state: PointerState,
    pub(super) buttons: MouseButtons,
    pub(super) scroll_drag: Option<ScrollDrag>,
    pub(super) file_target: Option<NodeId>,
    pub(super) shown: kivact_types::Cursor,
}

impl Default for Pointer {
    fn default() -> Self {
        Self {
            state: PointerState::default(),
            buttons: MouseButtons::default(),
            scroll_drag: None,
            file_target: None,
            shown: kivact_types::Cursor::Default,
        }
    }
}

impl Pointer {
    /// The mouse's window position while it is in the window.
    pub(crate) fn mouse(&self) -> Option<Point> {
        self.state.mouse
    }

    /// Note `cursor` as shown; whether it differs from the one shown.
    pub(crate) fn show(&mut self, cursor: kivact_types::Cursor) -> bool {
        std::mem::replace(&mut self.shown, cursor) != cursor
    }

    /// Drop removed and leaving nodes: a scrollbar drag of one ends, and
    /// a file drag over one is over nothing.
    pub(crate) fn forget_missing(&mut self, tree: &NodeTree) {
        self.state.forget_missing(tree);
        if self.scroll_drag.is_some_and(|d| !tree.live(d.node)) {
            self.scroll_drag = None;
        }
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

/// Single-touch scroll candidacy, retained only until cancellation or multi-touch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TouchScroll {
    pub(super) start: Point,
    pub(super) last: Point,
    pub(super) from: NodeId,
    pub(super) origin: NodeId,
    pub(super) modal: Option<NodeId>,
    /// None until takeover; true is vertical, false horizontal.
    pub(super) vertical: Option<bool>,
}

/// What the state machine follows of one pointer.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Track {
    /// Which pointer it is.
    pub(crate) source: PointerSource,
    /// The window position of its last event.
    pub(crate) at: Point,
    /// The hit node and its ancestors; always empty for a pointer that is
    /// not primary (it hovers nothing).
    pub(crate) hover: Vec<NodeId>,
    /// The hover chain at a primary press of a primary pointer, until the
    /// release.
    pub(crate) pressed: Vec<NodeId>,
    /// The listener that received the first `Down`, until all buttons
    /// release (a contact: until it ends).
    pub(crate) capture: Option<NodeId>,
    /// The nearest pointer listener under the pointer (`Enter`/`Leave`).
    pub(crate) over: Option<NodeId>,
    /// The exact activation target that received the primary press of a
    /// primary pointer.
    pub(crate) press_target: Option<FocusTarget>,
    pub(super) touch_scroll: Option<TouchScroll>,
    pub(super) touch_cancel_pending: bool,
}

impl Track {
    pub(crate) fn new(source: PointerSource) -> Self {
        Self {
            source,
            at: Point::default(),
            hover: Vec::new(),
            pressed: Vec::new(),
            capture: None,
            over: None,
            press_target: None,
            touch_scroll: None,
            touch_cancel_pending: false,
        }
    }
}

/// One pointer: the mouse, or a touch or pen contact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PointerSource {
    /// The pointer's id (the mouse is 1).
    pub(crate) id: PointerId,
    /// Mouse, touch or pen.
    pub(crate) kind: PointerKind,
    /// The pressure at its last event, when reported.
    pub(crate) pressure: Option<f32>,
    /// The pen's tilt at its last event, when reported.
    pub(crate) tilt: Option<(f32, f32)>,
    /// The buttons held after its last event (none after a `Cancel`).
    pub(crate) buttons: PointerButtons,
    /// Whether it is the primary pointer of its kind (Pointer Events'
    /// `isPrimary`): the mouse always; a touch or pen contact when no other
    /// contact of its kind was in contact at its `Down`, until it ends.
    pub(crate) primary: bool,
}

pub(crate) type CursorKind = kivact_wire::Cursor;

impl PointerSource {
    /// The mouse.
    pub(crate) const MOUSE: PointerSource = PointerSource {
        id: PointerId::MOUSE,
        kind: PointerKind::Mouse,
        pressure: None,
        tilt: None,
        buttons: PointerButtons::NONE,
        primary: true,
    };
}

#[derive(Clone, Copy)]
pub(crate) struct ScrollDrag {
    pub(crate) node: NodeId,
    pub(crate) vertical: bool,
    pub(crate) pointer: f32,
    pub(crate) offset: f32,
}

/// Mouse buttons currently reported down by the shell. A release clears only
/// its own bit; the representative Move button prefers a held primary button.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct MouseButtons(u8);

impl MouseButtons {
    pub(crate) fn set(&mut self, button: MouseButton, pressed: bool) {
        let bit = match button {
            MouseButton::Primary => 1,
            MouseButton::Secondary => 2,
            MouseButton::Middle => 4,
            MouseButton::Back => 8,
            MouseButton::Forward => 16,
        };
        if pressed {
            self.0 |= bit;
        } else {
            self.0 &= !bit;
        }
    }

    pub(crate) fn held(self) -> Option<MouseButton> {
        if self.0 & 1 != 0 {
            Some(MouseButton::Primary)
        } else if self.0 & 2 != 0 {
            Some(MouseButton::Secondary)
        } else if self.0 & 4 != 0 {
            Some(MouseButton::Middle)
        } else if self.0 & 8 != 0 {
            Some(MouseButton::Back)
        } else if self.0 & 16 != 0 {
            Some(MouseButton::Forward)
        } else {
            None
        }
    }

    pub(crate) fn clear(&mut self) {
        self.0 = 0;
    }
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
            for chain in [&mut track.hover, &mut track.pressed] {
                chain.retain(|n| tree.live(*n));
            }
            for slot in [&mut track.capture, &mut track.over] {
                if slot.is_some_and(|n| !tree.live(n)) {
                    *slot = None;
                }
            }
            if track
                .press_target
                .is_some_and(|target| !tree.live(target.node))
            {
                track.press_target = None;
            }
        }
    }
}
