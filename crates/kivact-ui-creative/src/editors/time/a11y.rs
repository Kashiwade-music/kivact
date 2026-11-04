//! Role: Time editor accessible item, lane and playhead projections.
use super::gesture::{State, Target};
use super::model::TimeItem;
use kivact::custom::part;
use kivact::draw::RegionId;
use kivact::interaction::{DragApi, ListGesture};
use kivact::state::IntoValue;

use kivact::view::Node;
use kivact::view::{A11yAction, A11yRequest, Role};
use std::rc::Rc;
#[derive(Clone, PartialEq)]
struct Info {
    id: RegionId,
    target: Target,
    name: String,
    value: String,
}

pub(crate) fn attach<T: TimeItem>(
    root: Node,
    state: State<T>,
    gesture: DragApi<()>,
    format: Rc<dyn Fn(f64) -> String>,
) -> Node {
    let data = state.data;
    let visible = state.visible;
    let scene = state.scene;
    let selection = state.selection;
    let choose = state.choose;
    let frame = state.playhead.into_value();
    let texts = crate::editors::shared::texts::Texts::new();
    let semantic_items = kivact::state::memo(move || {
        let mut result = Vec::new();
        if let Ok(scene) = scene.get() {
            for lane in visible.get() {
                if let Some(id) = scene.id(&Target::Lane(lane.id)) {
                    result.push(Info {
                        id,
                        target: Target::Lane(lane.id),
                        name: lane.label,
                        value: String::new(),
                    });
                }
            }
            for item in data.get() {
                for target in [
                    Target::Item(item.id()),
                    Target::Trim(item.id(), false),
                    Target::Trim(item.id(), true),
                ] {
                    if let Some(id) = scene.id(&target) {
                        let name = match target {
                            Target::Trim(_, right) => texts.trim(right, item.label()),
                            _ => item.label().to_owned(),
                        };
                        result.push(Info {
                            id,
                            target,
                            name,
                            value: texts.selection(selection.with(|s| s.contains(&item.id()))),
                        });
                    }
                }
            }
        }
        result
    });
    let marker =
        kivact::state::memo(move || scene.get().ok().and_then(|s| s.id(&Target::Playhead)));
    let root = root.regions(
        semantic_items,
        |info| info.id,
        move |_, info| {
            let target = info.get_untracked().target;
            // A press on a region drags from there.
            let result = {
                let input_part = part(|input| input);
                let drag_source = gesture.source(());
                kivact::custom::Part::new(input_part.state(), move |node| {
                    node.part(input_part)
                        .role(Role::Button)
                        .focusable(true)
                        .name(move || info.get().name)
                        .part(drag_source)
                })
            };
            match target {
                Target::Item(id) => kivact::custom::Part::new(result.state(), move |node| {
                    node.part(result)
                        .value_text(move || info.get().value)
                        .a11y_action(A11yAction::SetValue, move |event| {
                            if let A11yRequest::SetValue(text) = event.request
                                && let Some(time) = text.parse::<f64>().ok()
                            {
                                state.set_item_time(id, time);
                            }
                        })
                        .a11y_action(A11yAction::Increment, move |_| {
                            if let Some(time) = state.data.with(|items| {
                                items.iter().find(|i| i.id() == id).map(TimeItem::time)
                            }) {
                                state.set_item_time(id, time + state.step());
                            }
                        })
                        .a11y_action(A11yAction::Decrement, move |_| {
                            if let Some(time) = state.data.with(|items| {
                                items.iter().find(|i| i.id() == id).map(TimeItem::time)
                            }) {
                                state.set_item_time(id, time - state.step());
                            }
                        })
                        .a11y_action(A11yAction::Select, move |_| {
                            if state.drag_state().is_none() {
                                choose.select(id, ListGesture::Replace);
                            }
                        })
                        .a11y_action(A11yAction::Activate, move |_| {
                            if state.drag_state().is_none() && !selection.with(|s| s.contains(&id))
                            {
                                choose.select(id, ListGesture::Replace);
                            }
                        })
                }),
                Target::Trim(id, _) => kivact::custom::Part::new(result.state(), move |node| {
                    node.part(result)
                        .a11y_action(A11yAction::Activate, move |_| {
                            if state.drag_state().is_none() {
                                choose.select(id, ListGesture::Replace);
                            }
                        })
                }),
                Target::Lane(id) => kivact::custom::Part::new(result.state(), move |node| {
                    node.part(result)
                        .role(Role::Group)
                        .focusable(true)
                        .a11y_action(A11yAction::Activate, move |_| {
                            state.activate(Target::Lane(id))
                        })
                }),
                Target::Playhead => result,
            }
        },
    );
    let markers = kivact::state::memo(move || marker.get().into_iter().collect::<Vec<_>>());
    root.regions(
        markers,
        |id| *id,
        move |_, _| {
            let format = format.clone();
            {
                let input_part = part(|input| input);
                let drag_source = gesture.source(());
                kivact::custom::Part::new(input_part.state(), move |node| {
                    node.part(input_part)
                        .role(Role::Slider)
                        .focusable(true)
                        .name(move || texts.playhead())
                        .part(drag_source)
                        .value_now(frame)
                        .value_text(move || format(frame.get()))
                        .a11y_action(A11yAction::SetValue, move |event| {
                            if let A11yRequest::SetValue(text) = event.request
                                && let Some(value) =
                                    text.parse::<f64>().ok().filter(|v| v.is_finite())
                            {
                                state.set_playhead(value.max(0.0));
                            }
                        })
                        .a11y_action(A11yAction::Increment, move |_| {
                            state.set_playhead(frame.get() + state.step())
                        })
                        .a11y_action(A11yAction::Decrement, move |_| {
                            state.set_playhead((frame.get() - state.step()).max(0.0))
                        })
                })
            }
        },
    )
}
