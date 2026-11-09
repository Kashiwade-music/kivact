//! Role: TimeEditor: the shared view of the timeline and the dope sheet.

use crate::editors::shared::lane::LaneExtensions;
use crate::editors::shared::metrics::SIDEBAR;
use crate::editors::shared::model::{EditorContext, Lane};
use crate::editors::shared::ruler::Ruler;
use crate::editors::shared::selection;
use crate::editors::shared::targets;
use crate::editors::time::gesture::{State, Target};
use crate::editors::time::input::{key, pointer, wheel};
use crate::editors::time::layout::lane_rect;
use crate::editors::time::model::TimeItem;
use kivact::custom::NodeRefExt;
use kivact::custom::part;
use kivact::interaction::EditSession;
use kivact::interaction::ListItem;
use kivact::state::IntoValue;
use kivact::state::{Callback, Value};
use kivact::style::{ColorToken, Overflow, style};

use kivact::view::{A11yAction, IntoView, Role, View, node_ref};
use kivact::view::{Node, node};

pub(crate) struct TimeEditor<T: TimeItem> {
    pub(crate) root: Node,
    pub(crate) items: Value<Vec<T>>,
    pub(crate) state: crate::TimeState,
    // xtask-allow(state-fields): Independent setup options and callback; this builder owns no gesture lifecycle.
    pub(crate) lanes: Option<Value<Vec<Lane>>>,
    pub(crate) snap: Value<bool>,
    pub(crate) step: Value<f64>,
    pub(crate) ruler: Ruler,
    pub(crate) lane_height: f64,
    pub(crate) on_edit: Option<
        Callback<
            crate::editors::shared::model::EditorEvent<crate::editors::time::model::TimeRequest>,
        >,
    >,
    pub(crate) extensions: LaneExtensions,
}

impl<T: TimeItem> TimeEditor<T> {
    pub(crate) fn new(
        items: impl IntoValue<Vec<T>>,
        state: crate::TimeState,
        lane_height: f64,
    ) -> Self {
        Self {
            root: node().role(Role::Canvas).focusable(true).style(
                style()
                    .height(280)
                    .overflow(Overflow::Hidden)
                    .background(ColorToken::Surface),
            ),
            items: items.into_value(),
            state,
            lanes: None,
            snap: false.into_value(),
            step: 1.0.into_value(),
            ruler: Ruler::default(),
            lane_height,
            on_edit: None,
            extensions: LaneExtensions::default(),
        }
    }
}

impl<T: TimeItem> IntoView for TimeEditor<T> {
    fn into_view(self) -> View {
        let Prepared {
            root,
            state,
            ruler,
            cells,
            cells_width,
            extensions,
        } = self.prepare();
        let data = state.data;
        let selection = state.selection;
        let scale = state.scale;
        let bounds = state.bounds;
        let gesture = crate::editors::time::input::drag(state);
        let reference = node_ref();
        let availability = kivact::custom::part(|input| input);
        observe_inputs(state, reference, availability.state());
        let root = super::a11y::attach(root, state, gesture, ruler.format.clone());
        let root = root
            .node_ref(reference)
            .child(cells)
            .value_text(move || {
                format!(
                    "{} items; {} selected; zoom {:.0}%",
                    data.with(Vec::len),
                    selection.with(Vec::len),
                    scale.get().clamp(bounds.get().min(), bounds.get().max()) * 100.0
                )
            })
            .draw(move |draw, size| {
                super::paint::paint(draw, size, state, &ruler, cells_width.get())
            })
            .on_pointer(move |event| state.context_pointer(event))
            .on_key(move |event| state.context_key(&event))
            .on_pointer_default(move |event| pointer(state, event))
            .on_wheel_default(move |event| wheel(state, event))
            .part(availability)
            .part(gesture.source_with(move || state.source_snapshot()))
            // After the drag's sources, which take Escape during a drag.
            .part({
                let input_part = part(|input| input);
                kivact::custom::Part::new(input_part.state(), move |node| {
                    node.part(input_part)
                        .region_events()
                        .on_key(move |event| key(state, event))
                })
            });
        match extensions.menu {
            Some(menu) => menu.call((
                root.a11y_action(A11yAction::ShowMenu, move |event| {
                    state.cancel();
                    state.context_request(event.region);
                }),
                state.context.into_value(),
            )),
            None => root.into_view(),
        }
    }
}

struct Prepared<T: TimeItem> {
    root: Node,
    state: State<T>,
    ruler: Ruler,
    cells: View,
    cells_width: Value<f64>,
    extensions: LaneExtensions,
}
impl<T: TimeItem> TimeEditor<T> {
    fn prepare(self) -> Prepared<T> {
        let Self {
            root,
            items,
            state:
                crate::TimeState {
                    selection: selected,
                    playhead,
                    transform,
                },
            lanes,
            snap,
            step,
            ruler,
            lane_height,
            on_edit,
            extensions,
        } = self;
        ruler.report();
        let bounds = ruler.zoom;
        let min_duration = ruler.min_duration;
        let selection = selected.into_value();
        let source = items.into_value();
        let data = valid_items(source);
        let scale = kivact::state::memo(move || transform.get().scale);
        let offset = kivact::state::memo(move || transform.get().offset);
        let _frame = playhead.into_value();
        let visible = visible_lanes(data, lanes);
        let choose = time_selection(data, visible, selected);
        let scene = time_scene(data, visible, selection);
        let cells_width = extensions.width(SIDEBAR);
        let (cells, child_focus) = crate::editors::shared::lane::lane_cells(
            visible,
            &extensions,
            move |index| lane_rect(index, lane_height),
            true,
        );
        let edit = crate::editors::shared::model::observed(
            EditSession::new(move |snapshot| {
                crate::editors::time::edit::restore(playhead, transform, snapshot)
            }),
            crate::editors::shared::model::phase_observer(on_edit),
        );
        let (press, gesture) = crate::editors::shared::gesture::CanvasGesture::editor(edit, || {
            crate::editors::time::gesture::Snapshot::Items(Vec::new())
        });
        let state = State {
            data,
            on_edit,
            selected,
            selection,
            transform,
            scale,
            offset,
            playhead,
            visible,
            choose,
            scene,
            snap,
            step,
            bounds,
            min_duration,
            lane_height,
            active: kivact::state::signal(None),
            child_focus,
            context: kivact::state::signal(EditorContext::at_time(playhead.get_untracked())),
            lane_activate: extensions.lane_activate,
            item_activate: extensions.item_activate,
            press,
            gesture,
            edit,
        };
        Prepared {
            root,
            state,
            ruler,
            cells,
            cells_width,
            extensions,
        }
    }
}

fn observe_inputs<T: TimeItem>(
    state: State<T>,
    reference: kivact::view::NodeRef,
    availability_state: Value<kivact::custom::PartState>,
) {
    let focus = reference.focus_within();
    kivact::state::effect(move || {
        let focused = focus.get();
        let visible = reference.is_visible();
        let items = state.data.get();
        let lanes = state.visible.get();
        let missing = state.initial().is_some_and(|snapshot| match snapshot {
            crate::editors::time::gesture::Snapshot::Items(initial) => initial.iter().any(|v| {
                !items
                    .iter()
                    .any(|i| i.id() == v.0 && lanes.iter().any(|lane| lane.id == i.lane()))
            }),
            _ => false,
        });
        let input = availability_state.get();
        state.press.inputs_changed(
            focused,
            visible && !input.hidden,
            !input.disabled,
            !missing,
            move || state.cancel(),
        );
    });
}

fn visible_lanes<T: TimeItem>(
    data: Value<Vec<T>>,
    lanes: Option<Value<Vec<Lane>>>,
) -> Value<Vec<Lane>> {
    kivact::state::memo(move || {
        if let Some(lanes) = lanes {
            return lanes.get();
        }
        let mut result = Vec::<Lane>::new();
        for item in data.get() {
            if !result.iter().any(|lane| lane.id == item.lane()) {
                result.push(Lane::new(item.lane(), ""));
            }
        }
        result
    })
    .into_value()
}

type TimeScene = Value<Result<std::rc::Rc<targets::Scene<Target>>, kivact::UiError>>;
fn time_scene<T: TimeItem>(
    data: Value<Vec<T>>,
    visible: Value<Vec<Lane>>,
    selection: Value<Vec<u64>>,
) -> TimeScene {
    targets::targets(move || {
        let selected = selection.get();
        let mut targets = vec![Target::Playhead];
        targets.extend(visible.get().iter().map(|lane| Target::Lane(lane.id)));
        for item in data.get() {
            if !visible.with(|lanes| lanes.iter().any(|lane| lane.id == item.lane())) {
                continue;
            }
            targets.push(Target::Item(item.id()));
            if item.duration() > 0.0 && selected.contains(&item.id()) {
                targets.extend([
                    Target::Trim(item.id(), false),
                    Target::Trim(item.id(), true),
                ]);
            }
        }
        Ok(targets)
    })
}

fn time_selection<T: TimeItem>(
    data: Value<Vec<T>>,
    visible: Value<Vec<Lane>>,
    selected: kivact::state::Binding<Vec<u64>>,
) -> kivact::interaction::ListApi<u64> {
    let order = kivact::state::memo(move || {
        data.get()
            .into_iter()
            .filter(|item| visible.with(|lanes| lanes.iter().any(|lane| lane.id == item.lane())))
            .map(|item| ListItem::new(item.id(), item.label()))
            .collect::<Vec<_>>()
    });
    selection::controller(order, selected)
}

fn valid_items<T: TimeItem>(items: Value<Vec<T>>) -> Value<Vec<T>> {
    kivact::state::memo(move || {
        items
            .get()
            .into_iter()
            .filter(TimeItem::valid)
            .collect::<Vec<_>>()
    })
    .into_value()
}
