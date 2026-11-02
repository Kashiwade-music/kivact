//! Role: Node edit proposals and accessible operations.
use crate::editors::node::gesture::Target;
use crate::editors::node::gesture::{DragMode, NodeMode, State};
use crate::editors::node::layout as geometry;
use crate::editors::node::model::{NodeControl, NodeControlValue, NodeData, NodeLink, NodeRequest};
use kivact::Value;
use kivact::draw::Point;
use kivact::interaction::ListGesture;
use kivact::ui::EditPhase;
use kivact::view::IntoValue;

pub(crate) fn same_slot(a: &NodeControl, b: &NodeControl) -> bool {
    if a.label != b.label || a.offset != b.offset {
        return false;
    }
    match (&a.value, &b.value) {
        (
            NodeControlValue::Number {
                min: a,
                max: b,
                step: c,
                ..
            },
            NodeControlValue::Number {
                min: x,
                max: y,
                step: z,
                ..
            },
        ) => a == x && b == y && c == z,
        (
            NodeControlValue::Choice { options: a, .. },
            NodeControlValue::Choice { options: b, .. },
        ) => a == b,
        (NodeControlValue::Toggle(_), NodeControlValue::Toggle(_)) => true,
        _ => false,
    }
}
impl State {
    pub(crate) fn request(self, request: NodeRequest) {
        if let Some(observer) = self.on_edit {
            observer.call(crate::editors::shared::model::EditorEvent::Request(request));
        }
    }
    pub(crate) fn data(self) -> Value<Vec<NodeData>> {
        self.nodes.into_value()
    }
    pub(crate) fn point(self, point: Point) -> Point {
        self.transform.get_untracked().to_content(point)
    }
    pub(crate) fn busy(self) -> bool {
        self.blocked.get_untracked()
            || self.zoom.is_gesturing().get_untracked()
            || self.edit.is_active()
            || self.drag_state().is_some()
    }
    /// Stop the drag without a callback, and a control's edit from the
    /// parameter popup, canceling them.
    pub(crate) fn cancel(self) {
        self.gesture.cancel();
        self.edit.cancel();
        self.reset();
    }
    /// Forget the gesture of a drag that was canceled.
    pub(crate) fn reset(self) {
        self.press.cancel(|| self.reset_fields());
    }
    pub(crate) fn reset_fields(self) {
        self.mode.set(NodeMode::Idle);
        self.active.set(None);
    }
    pub(crate) fn select(self, id: u64, gesture: ListGesture) {
        self.selected_link.set(None);
        self.active.set(None);
        self.choose.select(id, gesture);
    }
    pub(crate) fn control(self, id: u64, index: usize) -> Option<NodeControl> {
        self.data().with_untracked(|nodes| {
            nodes
                .iter()
                .find(|n| n.id == id)
                .and_then(|n| n.controls.get(index))
                .cloned()
        })
    }
    /// The control as it was when the drag pressed it or the popup opened or
    /// began its edit, which an edit changes from.
    pub(crate) fn slot_of(self, id: u64, index: usize) -> Option<NodeControl> {
        let pressed = self
            .press
            .with_payload(|payload| match payload {
                DragMode::Control {
                    node,
                    index: at,
                    slot,
                } if *node == id && *at == index => Some(slot.clone()),
                _ => None,
            })
            .flatten();
        pressed.or_else(|| {
            self.mode.with_untracked(|mode| match mode {
                NodeMode::Popup {
                    node,
                    index: at,
                    slot,
                    ..
                } if *node == id && *at == index => Some(slot.clone()),
                _ => None,
            })
        })
    }
    /// The control whose original the drag or the popup holds, with its
    /// position. Tracked.
    pub(crate) fn held_slot(self) -> Option<(u64, usize, NodeControl)> {
        let pressed = self
            .press
            .with_payload(|payload| match payload {
                DragMode::Control { node, index, slot } => Some((*node, *index, slot.clone())),
                _ => None,
            })
            .flatten();
        pressed.or_else(|| {
            self.mode.with(|mode| match mode {
                NodeMode::Popup {
                    node, index, slot, ..
                } => Some((*node, *index, slot.clone())),
                _ => None,
            })
        })
    }
    /// Whether the popup of the control is open. Tracked.
    pub(crate) fn popup_is(self, id: u64, index: usize) -> bool {
        self.mode.with(|mode| {
            matches!(mode, NodeMode::Popup { node, index: at, .. } if *node == id && *at == index)
        })
    }
    /// Close the parameter popup, if one is open.
    pub(crate) fn close_popup(self) {
        self.mode.update(|mode| {
            if matches!(mode, NodeMode::Popup { .. }) {
                *mode = NodeMode::Idle;
            }
        });
    }
    pub(crate) fn change_raw(self, id: u64, index: usize, value: Option<f64>, delta: f64) {
        let original = self.slot_of(id, index).or_else(|| self.control(id, index));
        if let Some(mut control) = original {
            control.change(value, delta);
            self.request(NodeRequest::Control {
                node: id,
                index,
                value: control.value,
            });
        }
    }
    pub(crate) fn change(self, id: u64, index: usize, value: Option<f64>, delta: f64) {
        if self.busy() || value.is_some_and(|v| !v.is_finite()) {
            return;
        }
        if let Some(mut control) = self.control(id, index) {
            control.change(value, delta);
            self.edit.once((), || {
                self.request(NodeRequest::Control {
                    node: id,
                    index,
                    value: control.value,
                });
            });
        }
    }
    pub(crate) fn control_phase(self, id: u64, index: usize, phase: EditPhase) {
        match phase {
            EditPhase::Begin if !self.busy() && self.popup_is(id, index) => {
                if let Some(control) = self.control(id, index) {
                    self.mode.update(|mode| {
                        if let NodeMode::Popup { slot, .. } = mode {
                            *slot = control.clone();
                        }
                    });
                    self.edit.begin(());
                }
            }
            EditPhase::Commit if self.popup_is(id, index) && self.edit.is_active() => {
                self.edit.commit();
            }
            EditPhase::Cancel if self.popup_is(id, index) && self.edit.is_active() => {
                self.edit.cancel();
            }
            _ => {}
        }
    }
    pub(crate) fn open(self, id: u64, index: usize) {
        if self.busy() {
            return;
        }
        let Some(control) = self.control(id, index) else {
            return;
        };
        if let Some(r) = self.reference.rect().get_untracked()
            && let Some(local) = self.data().with_untracked(|nodes| {
                nodes
                    .iter()
                    .find(|n| n.id == id)
                    .and_then(|n| geometry::control(n, index))
            })
        {
            let p = self
                .transform
                .get_untracked()
                .to_view(Point::new(local.x, local.y + local.height));
            self.mode.set(NodeMode::Popup {
                node: id,
                index,
                at: Point::new(r.x + p.x, r.y + p.y),
                slot: control,
            });
        }
    }
    pub(crate) fn activate(self, id: u64, index: usize) {
        if self.busy() {
            return;
        }
        self.active.set(Some((id, index)));
        match self.control(id, index).map(|c| c.value) {
            Some(NodeControlValue::Choice { .. }) => self.open(id, index),
            Some(NodeControlValue::Toggle(_)) => self.change(id, index, None, 1.0),
            _ => {}
        }
    }
    pub(crate) fn valid_link(self, link: NodeLink) -> bool {
        link.from != link.to
            && geometry::endpoints(&self.nodes.get_untracked(), link).is_some()
            && (self.can_connect.get_untracked())(&link)
    }
    pub(crate) fn links_edit(self, remove: Option<NodeLink>, add: Option<NodeLink>) {
        if self.busy() {
            return;
        }
        self.write_links(remove, add);
    }
    /// Write one edit that removes `remove` and adds `add` (when valid),
    /// whatever else is going on: the end of a link drag, which is itself
    /// the gesture that makes the editor busy.
    pub(crate) fn write_links(self, remove: Option<NodeLink>, add: Option<NodeLink>) {
        let add = add.filter(|link| self.valid_link(*link));
        if remove.is_none() && add.is_none() {
            return;
        }
        self.edit.once((), || {
            if let Some(link) = remove {
                self.request(NodeRequest::Disconnect(link));
            }
            if let Some(link) = add {
                self.request(NodeRequest::Connect(link));
            }
        });
    }
    pub(crate) fn connect(self, from: u64, output: usize, to: u64, input: usize) {
        self.links_edit(None, Some(NodeLink::new(from, output, to, input)));
    }
    pub(crate) fn invoke(self, target: Target, select: bool) {
        if self.busy() {
            return;
        }
        match target {
            Target::Item(id) => {
                if select || !self.selected.get_untracked().contains(&id) {
                    self.select(id, ListGesture::Replace);
                }
            }
            Target::Control(id, index, _) => self.activate(id, index),
            Target::Link(..) => {
                self.selected_link.set(target.as_link());
                self.choose.clear();
            }
            Target::Socket(id, true, index) => self.mode.set(NodeMode::Connecting {
                from: id,
                output: index,
            }),
            Target::Socket(id, false, index) => {
                if let NodeMode::Connecting { from, output } = self.mode.get_untracked() {
                    self.connect(from, output, id, index);
                    self.mode.set(NodeMode::Idle);
                }
            }
        }
    }
}
