//! Role: Transaction ownership and selective restoration for time editors.
use crate::editors::time::gesture::{Snapshot, State};
use crate::editors::time::model::TimeItem;
use kivact::geometry::Point;
use kivact::state::Binding;
pub(crate) fn restore(
    playhead: Binding<f64>,
    zoom: Binding<f64>,
    pan: Binding<Point>,
    snapshot: Snapshot,
) {
    match snapshot {
        Snapshot::Items(..) => {}
        Snapshot::Playhead(value) => playhead.set(value),
        Snapshot::View(z, p) => {
            zoom.set(z);
            pan.set(p);
        }
    }
}
impl<T: TimeItem> State<T> {
    pub(crate) fn request(self, request: crate::editors::time::model::TimeRequest) {
        if let Some(callback) = self.on_edit {
            callback.call(crate::editors::shared::model::EditorEvent::Request(request));
        }
    }

    pub(crate) fn set_item_time(self, id: u64, time: f64) {
        if !time.is_finite() || self.drag_state().is_some() {
            return;
        }
        if let Some(item) = self
            .data
            .with(|items| items.iter().find(|i| i.id() == id).cloned())
            && item.time() != time
        {
            self.edit.once(self.snapshot(&[id]), || {
                self.request(crate::editors::time::model::TimeRequest::Retime(vec![
                    crate::editors::shared::model::ItemTime {
                        id,
                        time,
                        duration: item.duration(),
                    },
                ]));
            });
        }
    }
    pub(crate) fn selected_ids(self) -> Vec<u64> {
        let selected = self.selection.get_untracked();
        self.data.with_untracked(|items| {
            items
                .iter()
                .filter(|item| selected.contains(&item.id()))
                .map(TimeItem::id)
                .collect()
        })
    }
    pub(crate) fn snapshot(self, ids: &[u64]) -> Snapshot {
        Snapshot::Items(
            self.data
                .get_untracked()
                .iter()
                .filter(|i| ids.contains(&i.id()))
                .map(|i| (i.id(), i.time(), i.duration(), i.lane()))
                .collect(),
        )
    }
    pub(crate) fn view_snapshot(self) -> Snapshot {
        Snapshot::View(self.scale.get_untracked(), self.offset.get_untracked())
    }
    /// Stop the drag without a callback, canceling its edit.
    pub(crate) fn cancel(self) {
        self.gesture.cancel();
        self.reset();
    }
    /// Forget the gesture of a drag that ended.
    pub(crate) fn reset(self) {
        self.press.cancel(|| self.reset_fields());
    }
    pub(crate) fn reset_fields(self) {}
    pub(crate) fn set_playhead(self, value: f64) {
        if !value.is_finite() || self.drag_state().is_some() {
            return;
        }
        let old = self.playhead.get_untracked();
        if old != value {
            self.edit.once(Snapshot::Playhead(old), || {
                self.playhead.set(value);
            });
        }
    }
    pub(crate) fn time(self, x: f64) -> f64 {
        (x - crate::editors::shared::metrics::SIDEBAR - self.offset.get_untracked().x)
            / self
                .scale
                .get_untracked()
                .clamp(self.bounds.get().min(), self.bounds.get().max())
    }
}
