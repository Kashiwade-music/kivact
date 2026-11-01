//! Role: Domain projection for graph and dope edits of the same animation.
use super::common::bind;
use kivact::Binding;
use kivact::prelude::{Signal, rgb};
use kivact_ui_creative as ui;
use std::collections::{HashMap, HashSet};
const ROWS: [&str; 8] = [
    "Summary",
    "Cube",
    "Object Transforms",
    "X Location",
    "Y Location",
    "Z Location",
    "Camera",
    "Focal Length",
];
fn includes(row: u64, curve: u64) -> bool {
    match row {
        0 => true,
        1 | 2 => curve < 3,
        3..=5 => curve == row - 3,
        _ => curve == 3,
    }
}
fn projected(curves: &[ui::Curve]) -> Vec<ui::Keyframe> {
    let mut result = Vec::new();
    for (row, label) in ROWS.iter().enumerate() {
        let mut keys = curves
            .iter()
            .filter(|curve| includes(row as u64, curve.id))
            .flat_map(|curve| &curve.keys)
            .collect::<Vec<_>>();
        keys.sort_by(|a, b| a.time.total_cmp(&b.time));
        keys.dedup_by(|a, b| (a.time - b.time).abs() < 0.001);
        for key in keys {
            result.push(
                ui::Keyframe::new(
                    key.id * 16 + row as u64 + 1,
                    row as u64,
                    key.time,
                    format!("{label} · frame {}", key.time),
                )
                .color(rgb(0xc6c8d0)),
            );
        }
    }
    result
}
fn members(key: &ui::Keyframe, curves: &[ui::Curve]) -> Vec<u64> {
    curves
        .iter()
        .filter(|curve| includes(key.lane, curve.id))
        .flat_map(|curve| {
            curve
                .keys
                .iter()
                .filter(|point| (point.time - key.time).abs() < 0.001)
                .map(|point| point.id)
        })
        .collect()
}
/// Grouping and summary rows are domain projections: one displayed key can
/// represent several channels at the same time. No screen coordinates enter it.
pub(super) fn bindings(
    curves: Signal<Vec<ui::Curve>>,
    selection: Signal<Vec<u64>>,
) -> (Binding<Vec<ui::Keyframe>>, Binding<Vec<u64>>) {
    let read = move || projected(&curves.get());
    let keys = bind(read, move |next: Vec<ui::Keyframe>| {
        let mut data = curves.get_untracked();
        let previous = projected(&data);
        let mut removed = HashSet::new();
        let mut changed = HashMap::new();
        for key in &previous {
            if let Some(new) = next.iter().find(|new| new.id == key.id) {
                if (new.time - key.time).abs() > 0.001 {
                    for id in members(key, &data) {
                        changed.insert(id, new.time);
                    }
                }
            } else {
                removed.extend(members(key, &data));
            }
        }
        for curve in &mut data {
            curve.keys.retain(|key| !removed.contains(&key.id));
            for key in &mut curve.keys {
                if let Some(time) = changed.get(&key.id) {
                    key.time = *time;
                }
            }
        }
        let mut next_id = data
            .iter()
            .flat_map(|curve| &curve.keys)
            .map(|key| key.id)
            .max()
            .unwrap_or(0)
            + 1;
        for key in next
            .iter()
            .filter(|key| !previous.iter().any(|old| old.id == key.id))
        {
            let channel = match key.lane {
                3..=5 => key.lane - 3,
                6 | 7 => 3,
                _ => 0,
            };
            if let Some(curve) = data.iter_mut().find(|curve| curve.id == channel) {
                curve.keys.push(ui::CurveKey::new(next_id, key.time, 0.0));
                next_id += 1;
            }
        }
        curves.set(data);
    });
    let selected = bind(
        move || {
            let data = curves.get();
            let selected = selection.get();
            read()
                .into_iter()
                .filter(|key| members(key, &data).iter().any(|id| selected.contains(id)))
                .map(|key| key.id)
                .collect()
        },
        move |selected: Vec<u64>| {
            let data = curves.get_untracked();
            let mut ids = read()
                .into_iter()
                .filter(|key| selected.contains(&key.id))
                .flat_map(|key| members(&key, &data))
                .collect::<Vec<_>>();
            ids.sort_unstable();
            ids.dedup();
            selection.set(ids);
        },
    );
    (keys, selected)
}

/// Apply graph requests to the catalog animation model.
pub(super) fn apply_graph(curves: &mut Vec<ui::Curve>, request: &ui::GraphRequest) {
    match request {
        ui::GraphRequest::Move(values) => {
            for key in curves.iter_mut().flat_map(|curve| &mut curve.keys) {
                if let Some(value) = values.iter().find(|value| value.id == key.id) {
                    key.time = value.time;
                    key.value = value.value;
                    key.in_tangent = value.in_tangent;
                    key.out_tangent = value.out_tangent;
                }
            }
        }
        ui::GraphRequest::Delete(ids) => {
            for curve in curves {
                curve.keys.retain(|key| !ids.contains(&key.id));
            }
        }
        ui::GraphRequest::Interpolate(ids, interpolation) => {
            for key in curves.iter_mut().flat_map(|curve| &mut curve.keys) {
                if ids.contains(&key.id) {
                    key.interpolation = *interpolation;
                }
            }
        }
        // Variants added later fall back to being ignored.
        _ => {}
    }
}
pub(super) fn graph_edits(
    curves: Signal<Vec<ui::Curve>>,
) -> impl Fn(ui::EditorEvent<ui::GraphRequest>) + 'static {
    let initial = kivact::signal(None);
    move |event| match event {
        ui::EditorEvent::Begin => initial.set(Some(curves.get_untracked())),
        ui::EditorEvent::Request(request) => curves.update(|curves| apply_graph(curves, &request)),
        ui::EditorEvent::Commit => initial.set(None),
        ui::EditorEvent::Cancel => {
            if let Some(snapshot) = initial.get_untracked() {
                curves.set(snapshot);
            }
            initial.set(None);
        }
        // Variants added later fall back to being ignored.
        _ => {}
    }
}
