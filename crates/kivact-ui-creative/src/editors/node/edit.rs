//! Role: Node edit proposals and accessible operations.
use crate::editors::node::gesture::target::Target;
use crate::editors::node::gesture::{Domain, Snapshot, State};
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
            || self.pan_active.get_untracked()
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
        self.released.set(None);
        self.rewire.set(None);
        self.popup.set(None);
        self.active.set(None);
        self.slot.set(None);
        self.domain.set(None);
        self.space.set(false);
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
    pub(crate) fn slot(self, id: u64, index: usize) -> Option<NodeControl> {
        let control = self.control(id, index)?;
        self.slot.set(Some((id, index, control.clone())));
        Some(control)
    }
    pub(crate) fn change_raw(self, id: u64, index: usize, value: Option<f64>, delta: f64) {
        let original = if self.domain.get() == Some(Domain::Control(id, index)) {
            self.slot
                .get_untracked()
                .filter(|(owner, at, _)| *owner == id && *at == index)
                .map(|(_, _, control)| control)
        } else {
            self.control(id, index)
        };
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
        if let Some(mut control) = self.slot(id, index) {
            let snapshot = Snapshot::Control(id, index, control.value.clone());
            control.change(value, delta);
            self.edit.once(snapshot, || {
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
            EditPhase::Begin if !self.busy() => {
                if let Some(control) = self.slot(id, index) {
                    self.domain.set(Some(Domain::Control(id, index)));
                    self.edit.begin(Snapshot::Control(id, index, control.value));
                }
            }
            EditPhase::Commit if self.domain.get() == Some(Domain::Control(id, index)) => {
                self.edit.commit();
                self.domain.set(None);
            }
            EditPhase::Cancel if self.domain.get() == Some(Domain::Control(id, index)) => {
                self.edit.cancel();
                self.domain.set(None);
            }
            _ => {}
        }
    }
    pub(crate) fn open(self, id: u64, index: usize) {
        if self.busy() {
            return;
        }
        self.slot(id, index);
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
            self.popup
                .set(Some((id, index, Point::new(r.x + p.x, r.y + p.y))));
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
            _ => {
                self.slot(id, index);
            }
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
        let add = add.filter(|link| self.valid_link(*link));
        if remove.is_none() && add.is_none() {
            return;
        }
        self.edit.once(Snapshot::Empty, || {
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
            Target::Socket(id, true, index) => self.port.set(Some((id, index))),
            Target::Socket(id, false, index) => {
                if let Some((from, output)) = self.port.get_untracked() {
                    self.connect(from, output, id, index);
                    self.port.set(None);
                }
            }
        }
    }
}
