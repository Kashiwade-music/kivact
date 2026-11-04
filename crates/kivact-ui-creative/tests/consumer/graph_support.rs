//! Role: Consumer-owned graph requests and rollback used by editor window tests.
use kivact::state::callback;
use kivact::state::signal;
use kivact::view::EditPhase;
use kivact_ui_creative as ui;
pub(super) fn apply_graph(curves: &mut Vec<ui::Curve>, request: &ui::GraphRequest) {
    match request {
        ui::GraphRequest::Move(moves) => {
            for key in curves.iter_mut().flat_map(|curve| &mut curve.keys) {
                if let Some(value) = moves.iter().find(|value| value.id == key.id) {
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
    }
}
pub(super) fn graph_edits<R: kivact::view::HandlerResult>(
    curves: impl Into<kivact::state::Binding<Vec<ui::Curve>>>,
    observer: impl Fn(EditPhase) -> R + 'static,
) -> impl Fn(ui::EditorEvent<ui::GraphRequest>) + 'static {
    let curves = curves.into();
    let observer = callback(observer);
    let initial = signal(None::<Vec<ui::Curve>>);
    let moved = signal(Vec::<u64>::new());
    move |event| match event {
        ui::EditorEvent::Begin => {
            initial.set(Some(curves.get_untracked()));
            moved.set(Vec::new());
            observer.call(EditPhase::Begin);
        }
        ui::EditorEvent::Request(request) => {
            if let ui::GraphRequest::Move(values) = &request {
                moved.update(|ids| ids.extend(values.iter().map(|value| value.id)));
            }
            let mut next = curves.get_untracked();
            apply_graph(&mut next, &request);
            curves.set(next);
        }
        ui::EditorEvent::Commit => {
            initial.set(None);
            observer.call(EditPhase::Commit);
        }
        ui::EditorEvent::Cancel => {
            // This application restores all proposed fields on affected surviving keys.
            // Rollback scope belongs to the consumer; unrelated data and deletions survive.
            if let Some(initial) = initial.get_untracked() {
                let ids = moved.get_untracked();
                let values = initial
                    .iter()
                    .flat_map(|curve| &curve.keys)
                    .filter(|key| ids.contains(&key.id))
                    .map(|key| ui::KeyMove {
                        id: key.id,
                        time: key.time,
                        value: key.value,
                        in_tangent: key.in_tangent,
                        out_tangent: key.out_tangent,
                    })
                    .collect();
                let mut next = curves.get_untracked();
                apply_graph(&mut next, &ui::GraphRequest::Move(values));
                curves.set(next);
            }
            initial.set(None);
            observer.call(EditPhase::Cancel);
        }
    }
}
pub(super) fn app_graph(
    curves: impl Into<kivact::state::Binding<Vec<ui::Curve>>>,
    state: ui::GraphState,
) -> ui::GraphEditor {
    let curves = curves.into();
    ui::graph_editor(curves, state).on_edit(graph_edits(curves, |_| {}))
}
