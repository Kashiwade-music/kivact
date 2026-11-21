//! Role: The graph component and the curve canvas view it builds.

use crate::editors::graph::gesture::{Settings, State};
use crate::editors::graph::model::{Curve, GraphRequest};
use crate::editors::shared::lane::{
    EditorMenu, GRAPH_LANE_WIDTH, LaneCell, LaneExtensions, graph_lane_y,
};
use crate::editors::shared::model::{EditorContext, EditorEvent, Lane};
use crate::editors::shared::ruler::{Ruler, RulerTicks};
use kivact::component;
use kivact::custom::part;
use kivact::interaction::ListItem;
use kivact::interaction::{EditSession, use_space_pan};
use kivact::state::{Callback, IntoValue, Value, signal};
use kivact::style::{Overflow, style};
use kivact::view::{A11yAction, Node, Role, View, node, node_ref};
use std::rc::Rc;

/// An editable graph whose curves and view intervals use application domain units.
/// Curves are read-only: `on_edit` reports requests for the application to apply.
/// Drag requests are absolute from the initial keys; ignored requests do not move them.
/// Key IDs must be unique across all curves. Selection does not emit edit phases.
///
/// Plot application curves without converting their time or value coordinates to pixels.
///
/// `state` holds the shown time and value ranges (an invalid axis draws with
/// its default range), the selected key IDs and the playhead, in application
/// time units. `lanes` displays only the curves whose IDs appear in it; an
/// explicit empty list hides all. `snap` rounds edited positions to the
/// steps when enabled. `time_step` and `value_step` set the keyboard and
/// snapping increments; a value that is not finite and positive falls back
/// to its default (1 and 0.01).
#[component]
pub fn graph_editor(
    curves: Value<Vec<Curve>>,
    state: crate::GraphState,
    lanes: Option<Value<Vec<Lane>>>,
    #[prop(default = false)] snap: Value<bool>,
    #[prop(default = 1.0)] time_step: Value<f64>,
    #[prop(default = 0.01)] value_step: Value<f64>,
    time_format: Option<Rc<dyn Fn(f64) -> String>>,
    value_format: Option<Rc<dyn Fn(f64) -> String>>,
    ruler_ticks: Option<RulerTicks>,
    on_edit: Option<Callback<EditorEvent<GraphRequest>>>,
    #[prop(default = Vec::new())] lane_cells: Vec<LaneCell>,
    on_lane_activate: Option<Callback<u64>>,
    on_activate: Option<Callback<u64>>,
    context_menu: Option<EditorMenu>,
) -> impl kivact::view::Element {
    let mut ruler = Ruler {
        format: Rc::new(|v| v.to_string()),
        ..Ruler::default()
    };
    if let Some(format) = time_format {
        ruler.format = format;
    }
    if let Some(ticks) = ruler_ticks {
        ruler.ticks = ticks;
    }
    let settings = Settings {
        ruler,
        value_format: value_format.unwrap_or_else(|| Rc::new(|v| v.to_string())),
        step: (time_step, value_step),
        cells_width: 0.0.into_value(),
    };
    let extensions = LaneExtensions {
        cells: lane_cells
            .into_iter()
            .map(|cell| (cell.width, cell.render))
            .collect(),
        lane_activate: on_lane_activate,
        item_activate: on_activate,
        menu: context_menu.map(|menu| menu.0),
    };
    GraphCanvas {
        curves,
        state,
        lanes,
        snap,
        settings,
        on_edit,
        extensions,
    }
    .into_node()
}

/// The settings a component collects, which build the curve canvas.
struct GraphCanvas {
    curves: Value<Vec<Curve>>,
    state: crate::GraphState,
    lanes: Option<Value<Vec<Lane>>>,
    snap: Value<bool>,
    settings: Settings,
    on_edit: Option<Callback<EditorEvent<GraphRequest>>>,
    extensions: LaneExtensions,
}

/// The focusable canvas that the graph's regions, pointer and keys attach to.
fn canvas_root() -> Node {
    node().role(Role::Canvas).focusable(true).style(
        style()
            .height(280)
            .min_width(0)
            .overflow(Overflow::Hidden)
            .background(crate::color::ISLAND),
    )
}

impl GraphCanvas {
    fn into_node(self) -> Node {
        let Prepared {
            state,
            reference,
            settings,
            lanes,
            cells,
            extensions,
        } = self.prepare();
        let curves = state.curves;
        let selected = state.selected;
        let gesture = state.drag();
        let availability = kivact::custom::part(|input| input);
        state.press.watch_inputs(
            crate::editors::shared::gesture::inputs_ok(reference, availability.state()),
            move || state.cancel(),
        );
        let root = crate::editors::graph::a11y::attach(
            canvas_root()
                .value_text(move || {
                    format!(
                        "{} keys; {} selected",
                        curves.get().iter().map(|c| c.keys.len()).sum::<usize>(),
                        selected.get().len()
                    )
                })
                .node_ref(reference),
            state,
            gesture,
            settings.clone(),
            lanes,
        );
        let root = root
            .child(cells)
            .draw(move |draw, size| {
                crate::editors::graph::paint::paint(draw, size, state, &settings, lanes)
            })
            .on_pointer(move |event| state.context_pointer(event))
            .on_key(move |event| state.context_key(&event))
            .on_pointer_default(move |event| state.pointer(event))
            .part(availability)
            .part(gesture.source_with(move || state.source_snapshot()))
            // After the drag's sources, which take Escape during a drag.
            .part({
                let input_part = part(|input| input);
                kivact::custom::Part::new(input_part.state(), move |node| {
                    node.part(input_part)
                        .region_events()
                        .on_key(move |event| state.key(event))
                })
            })
            // Space arms the pan; it reaches ancestors like on any pan surface.
            .part(state.space.control())
            .on_wheel(move |event| state.wheel(event));
        match extensions.menu {
            Some(menu) => node().child(menu.call((
                root.a11y_action(A11yAction::ShowMenu, move |event| {
                    state.cancel();
                    state.context_request(event.region);
                }),
                state.context.into_value(),
            ))),
            None => root,
        }
    }
}

struct Prepared {
    state: State,
    reference: kivact::view::NodeRef,
    settings: Rc<Settings>,
    lanes: Value<Vec<Lane>>,
    cells: View,
    extensions: LaneExtensions,
}
impl GraphCanvas {
    fn prepare(self) -> Prepared {
        let Self {
            curves,
            state,
            lanes,
            snap,
            mut settings,
            on_edit,
            extensions,
        } = self;
        settings.cells_width = extensions.width(GRAPH_LANE_WIDTH);
        settings.ruler.report();
        let settings = Rc::new(settings);
        let crate::GraphState {
            view,
            selection: selected,
            playhead,
        } = state;
        let lanes = graph_lanes(lanes, curves);
        let visible = crate::editors::graph::a11y::visible(curves, lanes, settings.clone());
        let choose = graph_selection(visible, selected);
        let targets = crate::editors::graph::a11y::scene(curves, visible, selected, lanes);
        let scene = targets.scene();
        let active = targets.active();
        let (cells, child_focus) = crate::editors::shared::lane::lane_cells(
            lanes,
            &extensions,
            |index| {
                kivact::geometry::Rect::new(
                    0.0,
                    graph_lane_y(index),
                    GRAPH_LANE_WIDTH,
                    crate::editors::shared::metrics::KEY_LANE,
                )
            },
            false,
        );
        let reference = node_ref();
        let rect = reference.rect();
        let edit = crate::editors::shared::model::observed(
            EditSession::new(move |snapshot| {
                crate::editors::graph::edit::restore(view, playhead, snapshot)
            }),
            crate::editors::shared::model::phase_observer(on_edit),
        );
        let (press, gesture) = crate::editors::shared::gesture::CanvasGesture::editor(edit, || {
            crate::editors::graph::gesture::Snapshot::Keys(Vec::new())
        });
        let state = State {
            curves,
            on_edit,
            view,
            selected,
            playhead,
            visible,
            scene,
            choose,
            size: crate::editors::shared::model::canvas_size(rect),
            snap,
            step: settings.step,
            edit,
            press,
            gesture,
            active,
            space: use_space_pan(),
            lane_activate: extensions.lane_activate,
            item_activate: extensions.item_activate,
            child_focus,
            context: signal(EditorContext::at_time(playhead.get_untracked())),
        };
        Prepared {
            state,
            reference,
            settings,
            lanes,
            cells,
            extensions,
        }
    }
}

fn graph_lanes(lanes: Option<Value<Vec<Lane>>>, curves: Value<Vec<Curve>>) -> Value<Vec<Lane>> {
    lanes.unwrap_or_else(|| {
        kivact::state::memo(move || {
            curves
                .get()
                .iter()
                .map(|c| Lane::new(c.id, c.label.clone()))
                .collect()
        })
    })
}

fn graph_selection(
    visible: Value<Vec<crate::editors::graph::gesture::VisibleKey>>,
    selected: kivact::state::Signal<Vec<u64>>,
) -> kivact::interaction::ListApi<u64> {
    let order = kivact::state::memo(move || {
        visible
            .get()
            .into_iter()
            .map(|k| ListItem::new(k.key.id, k.name))
            .collect::<Vec<_>>()
    });
    crate::editors::shared::selection::controller(order, selected)
}
