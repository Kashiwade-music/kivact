//! Role: Graph target allocation and accessibility projections of domain values.
use crate::editors::graph::gesture::{Info, Settings, State, Target, VisibleKey};
use crate::editors::graph::model::Curve;
use crate::editors::graph::model::Interpolation;
use crate::editors::shared::model::Lane;
use kivact::custom::part;
use kivact::interaction::DragApi;
use kivact::state::{Binding, Value};
use kivact::view::Node;
use kivact::view::{A11yAction, A11yRequest, Role};
use std::rc::Rc;

pub(crate) fn visible(
    curves: Value<Vec<Curve>>,
    lanes: Value<Vec<Lane>>,
    settings: Rc<Settings>,
) -> Value<Vec<VisibleKey>> {
    kivact::state::memo(move || {
        let lanes = lanes.get();
        let mut result = Vec::new();
        for curve in curves
            .get()
            .into_iter()
            .filter(|c| lanes.iter().any(|l| l.id == c.id))
        {
            let mut keys = curve.keys;
            keys.sort_by(|a, b| a.time.total_cmp(&b.time));
            let mut previous_bezier = false;
            for key in keys {
                let name = format!("{} key {}", curve.label, (settings.ruler.format)(key.time));
                let bezier = key.interpolation == Interpolation::Bezier;
                result.push(VisibleKey {
                    curve: curve.id,
                    key,
                    name,
                    color: curve.color,
                    previous_bezier,
                });
                previous_bezier = bezier;
            }
        }
        result
    })
}
pub(crate) fn scene(
    curves: Value<Vec<Curve>>,
    visible: Value<Vec<VisibleKey>>,
    selected: Binding<Vec<u64>>,
    lanes: Value<Vec<Lane>>,
) -> crate::editors::shared::targets::Targets<Target, Target> {
    crate::editors::shared::targets::targets(move || {
        let mut unique = std::collections::HashSet::new();
        if curves
            .get()
            .iter()
            .flat_map(|c| &c.keys)
            .any(|k| !unique.insert(k.id))
        {
            return Err(kivact::UiError::msg("duplicate graph key ID"));
        }
        let mut result = vec![Target::Playhead];
        result.extend(lanes.get().iter().map(|l| Target::Lane(l.id)));
        for key in visible.get() {
            result.push(Target::Key(key.key.id));
            if selected.get().contains(&key.key.id) {
                if key.key.interpolation == Interpolation::Bezier {
                    result.push(Target::Tangent(key.key.id, false));
                }
                if key.key.interpolation == Interpolation::Bezier || key.previous_bezier {
                    result.push(Target::Tangent(key.key.id, true));
                }
            }
        }
        Ok(result)
    })
}
fn infos(state: State, settings: Rc<Settings>, lanes: Value<Vec<Lane>>) -> Value<Vec<Info>> {
    kivact::state::memo(move || {
        let Ok(scene) = state.scene.get() else {
            return Vec::new();
        };
        let mut result = Vec::new();
        let mut add = |target, name, value, role, selected| {
            if let Some(id) = scene.id(&target) {
                result.push(Info {
                    id,
                    target,
                    name,
                    value,
                    role,
                    selected,
                });
            }
        };
        add(
            Target::Playhead,
            "Playhead".into(),
            (settings.ruler.format)(state.playhead.get()),
            Role::Slider,
            false,
        );
        for lane in lanes.get() {
            add(
                Target::Lane(lane.id),
                lane.label,
                String::new(),
                Role::Group,
                false,
            );
        }
        for key in state.visible.get() {
            let selected = state.selected.get().contains(&key.key.id);
            add(
                Target::Key(key.key.id),
                key.name.clone(),
                format!(
                    "{}, {}, {:?}",
                    (settings.ruler.format)(key.key.time),
                    (settings.value_format)(key.key.value),
                    key.key.interpolation
                ),
                Role::Button,
                selected,
            );
            for incoming in [true, false] {
                let tangent = if incoming {
                    key.key.in_tangent
                } else {
                    key.key.out_tangent
                };
                add(
                    Target::Tangent(key.key.id, incoming),
                    format!(
                        "{} {} tangent",
                        key.name,
                        if incoming { "incoming" } else { "outgoing" }
                    ),
                    format!("{}, {}", tangent.time, tangent.value),
                    Role::Button,
                    selected,
                );
            }
        }
        result
    })
}
pub(crate) fn attach(
    root: Node,
    state: State,
    gesture: DragApi<crate::editors::graph::gesture::Snapshot>,
    settings: Rc<Settings>,
    lanes: Value<Vec<Lane>>,
) -> Node {
    root.regions(
        infos(state, settings, lanes),
        |info| info.id,
        move |_, info| {
            let target = info.get_untracked().target;
            // A press on a region drags from there.
            let mut result = {
                let input_part = part(|input| input);
                let drag_source = gesture.source_with(move || state.source_snapshot());
                kivact::custom::Part::new(input_part.state(), move |node| {
                    node.part(input_part)
                        .role(info.get_untracked().role)
                        .name(move || info.get().name)
                        .value_text(move || info.get().value)
                        .part(drag_source)
                })
            };
            if matches!(target, Target::Lane(_)) {
                return kivact::custom::Part::new(result.state(), move |node| {
                    node.part(result)
                        .focusable(true)
                        .a11y_action(A11yAction::Activate, move |_| state.activate(target))
                });
            }
            result = kivact::custom::Part::new(result.state(), move |node| {
                node.part(result).focusable(true)
            });
            if !matches!(target, Target::Playhead) {
                result = kivact::custom::Part::new(result.state(), move |node| {
                    node.part(result)
                        .selected(move || info.get().selected)
                        .a11y_action(A11yAction::Select, move |_| {
                            if state.drag_state().is_none() {
                                state.select(target, true);
                            }
                        })
                        .a11y_action(A11yAction::Activate, move |_| {
                            if state.drag_state().is_none() {
                                state.select(target, false);
                            }
                        })
                });
            }
            kivact::custom::Part::new(result.state(), move |node| {
                node.part(result)
                    .a11y_action(A11yAction::SetValue, move |event| {
                        if let A11yRequest::SetValue(value) = event.request {
                            let pair = if matches!(target, Target::Playhead) {
                                value
                                    .parse::<f64>()
                                    .ok()
                                    .filter(|v| v.is_finite())
                                    .map(|v| (v, 0.0))
                            } else {
                                crate::editors::graph::edit::pair(&value)
                            };
                            if let Some(pair) = pair {
                                state.change(target, Some(pair), 0.0);
                            }
                        }
                    })
                    .a11y_action(A11yAction::Increment, move |_| {
                        state.change(target, None, state.step().0)
                    })
                    .a11y_action(A11yAction::Decrement, move |_| {
                        state.change(target, None, -state.step().0)
                    })
            })
        },
    )
}
