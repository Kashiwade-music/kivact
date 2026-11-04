//! Role: Graph edit proposals and restoration of bound view values.

use crate::editors::graph::gesture::{Snapshot, State, Target};
use crate::editors::graph::layout::{Space, fit, number, snapped};
use crate::editors::graph::model::Interpolation;
use crate::editors::graph::model::{CurveTangent, GraphRequest, GraphView, KeyMove};
use kivact::interaction::ListGesture;
use kivact::state::Binding;

pub(crate) fn restore(view: Binding<GraphView>, playhead: Binding<f64>, snapshot: Snapshot) {
    match snapshot {
        Snapshot::View(initial) => view.set(initial),
        Snapshot::Playhead(initial) => playhead.set(initial),
        Snapshot::Keys(_) | Snapshot::Tangent(_, _) => {}
    }
}

impl State {
    pub(crate) fn geometry(self) -> Space {
        Space::new(self.view.get_untracked(), self.size.get_untracked())
    }
    pub(crate) fn target(self, region: Option<kivact::draw::RegionId>) -> Option<Target> {
        region.and_then(|id| {
            self.scene.with_untracked(|scene| {
                scene
                    .as_ref()
                    .ok()
                    .and_then(|scene| scene.target(id).copied())
            })
        })
    }
    pub(crate) fn ids(self) -> Vec<u64> {
        let selected = self.selected.get_untracked();
        self.visible.with_untracked(|keys| {
            keys.iter()
                .filter(|key| selected.contains(&key.key.id))
                .map(|key| key.key.id)
                .collect()
        })
    }
    fn moves(self, ids: &[u64]) -> Vec<KeyMove> {
        self.curves.with_untracked(|curves| {
            curves
                .iter()
                .flat_map(|c| &c.keys)
                .filter(|key| ids.contains(&key.id))
                .map(|key| KeyMove {
                    id: key.id,
                    time: key.time,
                    value: key.value,
                    in_tangent: key.in_tangent,
                    out_tangent: key.out_tangent,
                })
                .collect()
        })
    }
    pub(crate) fn request(self, request: GraphRequest) {
        if let Some(callback) = self.on_edit {
            callback.call(crate::editors::shared::model::EditorEvent::Request(request));
        }
    }
    pub(crate) fn positions(self, ids: &[u64]) -> Snapshot {
        Snapshot::Keys(self.moves(ids))
    }
    pub(crate) fn tangent_snapshot(self, id: u64, incoming: bool) -> Snapshot {
        Snapshot::Tangent(self.moves(&[id]), incoming)
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
    /// The time and value increments; an invalid one falls back to its default.
    pub(crate) fn step(self) -> (f64, f64) {
        let valid = |step: f64, default: f64| {
            if step.is_finite() && step > 0.0 {
                step
            } else {
                default
            }
        };
        (
            valid(self.step.0.get_untracked(), 1.0),
            valid(self.step.1.get_untracked(), 0.01),
        )
    }
    pub(crate) fn select(self, target: Target, replace: bool) {
        self.active.set(Some(target));
        if let Target::Key(id) | Target::Tangent(id, _) = target
            && (replace || !self.selected.get_untracked().contains(&id))
        {
            self.choose.select(id, ListGesture::Replace);
        }
    }
    // Proposals always derive from the pointer capture, even when requests are ignored.
    pub(crate) fn drag_fields(self, initial: &Snapshot, time: f64, value: f64) {
        self.propose(initial, time, value, self.snap.get_untracked());
    }
    pub(crate) fn propose(self, initial: &Snapshot, time: f64, value: f64, snap: bool) {
        let (keys, incoming) = match initial {
            Snapshot::Keys(keys) => (keys, None),
            Snapshot::Tangent(keys, incoming) => (keys, Some(*incoming)),
            _ => return,
        };
        let step = self.step();
        let keys = keys
            .iter()
            .copied()
            .map(|mut key| {
                if let Some(incoming) = incoming {
                    let tangent = if incoming {
                        &mut key.in_tangent
                    } else {
                        &mut key.out_tangent
                    };
                    tangent.time = snapped(number(tangent.time + time, tangent.time), step.0, snap);
                    tangent.value =
                        snapped(number(tangent.value + value, tangent.value), step.1, snap);
                } else {
                    key.time = snapped(number(key.time + time, key.time), step.0, snap);
                    key.value = snapped(number(key.value + value, key.value), step.1, snap);
                }
                key
            })
            .collect();
        self.request(GraphRequest::Move(keys));
    }
    pub(crate) fn change(self, target: Target, pair: Option<(f64, f64)>, delta: f64) {
        if self.drag_state().is_some() {
            return;
        }
        match target {
            Target::Playhead => {
                let old = self.playhead.get_untracked();
                let next = pair.map_or(old + delta, |pair| pair.0);
                if next.is_finite() {
                    self.edit.once(Snapshot::Playhead(old), || {
                        self.playhead.set(next);
                    });
                }
            }
            Target::Key(id) => {
                if !self
                    .visible
                    .with_untracked(|keys| keys.iter().any(|key| key.key.id == id))
                {
                    return;
                }
                let step = self.step();
                let mut keys = self.moves(&[id]);
                self.edit.once(Snapshot::Keys(keys.clone()), || {
                    for key in &mut keys {
                        let (time, value) = pair.unwrap_or((key.time + delta, key.value));
                        key.time =
                            snapped(number(time, key.time), step.0, self.snap.get_untracked());
                        key.value =
                            snapped(number(value, key.value), step.1, self.snap.get_untracked());
                    }
                    self.request(GraphRequest::Move(keys));
                });
            }
            Target::Tangent(id, incoming) => {
                let mut keys = self.moves(&[id]);
                self.edit
                    .once(Snapshot::Tangent(keys.clone(), incoming), || {
                        for key in &mut keys {
                            let tangent = if incoming {
                                &mut key.in_tangent
                            } else {
                                &mut key.out_tangent
                            };
                            let pair = pair.unwrap_or((tangent.time + delta, tangent.value));
                            *tangent = CurveTangent::new(
                                number(pair.0, tangent.time),
                                number(pair.1, tangent.value),
                            );
                        }
                        self.request(GraphRequest::Move(keys));
                    });
            }
            Target::Lane(_) => {}
        }
    }
    pub(crate) fn set_view(self, next: GraphView) {
        self.edit
            .once(Snapshot::View(self.view.get_untracked()), || {
                self.view.set(next);
            });
    }
    pub(crate) fn fit(self, selected: bool) {
        let selection = if selected {
            self.selected.get_untracked()
        } else {
            Vec::new()
        };
        if let Some(next) = self
            .visible
            .with_untracked(|keys| fit(keys, &selection, self.step()))
        {
            self.set_view(next);
        }
    }
    pub(crate) fn interpolate(self, interpolation: Interpolation) {
        let ids = self.ids();
        if ids.is_empty() {
            return;
        }
        self.edit.once(self.positions(&ids), || {
            self.request(GraphRequest::Interpolate(ids, interpolation));
        });
    }
    pub(crate) fn delete(self) {
        let ids = self.ids();
        if ids.is_empty() {
            return;
        }
        self.edit.once(self.positions(&ids), || {
            self.request(GraphRequest::Delete(ids));
        });
        self.choose.clear();
    }
}

pub(crate) fn pair(text: &str) -> Option<(f64, f64)> {
    let mut values = text
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty());
    let time: f64 = values.next()?.parse().ok()?;
    let value: f64 = values.next()?.parse().ok()?;
    (time.is_finite() && value.is_finite() && values.next().is_none()).then_some((time, value))
}
