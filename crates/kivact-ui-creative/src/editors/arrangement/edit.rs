//! Role: Arrangement edits and application request operations.
pub(crate) mod actions;
use super::gesture::{Snapshot, State};
use kivact::draw::Point;
/// Restores only bound scalar and view values; the application restores items.
pub(crate) fn restore(s: Restore, snapshot: Snapshot) {
    match snapshot {
        Snapshot::Clips(..) | Snapshot::Automation(..) => {}
        Snapshot::Playhead(v) => s.playhead.set(v),
        Snapshot::Loop(v) => s.loop_range.set(v),
        Snapshot::Cycle(v) => s.cycling.set(v),
        Snapshot::View(z, p) => {
            s.zoom.set(z);
            s.pan.set(p);
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) struct Restore {
    pub playhead: kivact::Binding<f64>,
    pub loop_range: kivact::Binding<(f64, f64)>,
    pub cycling: kivact::Binding<bool>,
    pub zoom: kivact::Binding<f64>,
    pub pan: kivact::Binding<Point>,
}
impl State {
    pub(crate) fn request(self, request: crate::editors::arrangement::model::ArrangementRequest) {
        if let Some(callback) = self.on_edit {
            callback.call(crate::editors::shared::model::EditorEvent::Request(request));
        }
    }

    pub(crate) fn snapshot(self, ids: &[u64]) -> Snapshot {
        Snapshot::Clips(
            self.data
                .get_untracked()
                .iter()
                .filter(|c| ids.contains(&c.id))
                .map(|c| (c.id, c.start, c.duration))
                .collect(),
        )
    }
    pub(crate) fn view_snapshot(self) -> Snapshot {
        Snapshot::View(self.pixels.get_untracked(), self.offset.get_untracked())
    }
    pub(crate) fn auto_snapshot(self, track: u64, id: u64) -> Option<Snapshot> {
        self.track_data.with_untracked(|tracks| {
            tracks.iter().find(|t| t.id == track).map(|t| {
                Snapshot::Automation(t.id, id, t.automation.iter().find(|p| p.id == id).cloned())
            })
        })
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
}
