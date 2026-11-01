//! Role: Public graph builder and reactive assembly of graph domain operations.
pub(crate) mod a11y;
pub(crate) mod edit;
pub(crate) mod gesture;
pub(crate) mod input;
pub(crate) mod layout;
pub(crate) mod model;
pub(crate) mod paint;
pub(crate) mod view_state;
use crate::editors::graph::gesture::{DragKind, Settings, State};
use crate::editors::graph::model::Curve;
use crate::editors::shared::lane::{GRAPH_LANE_WIDTH, LaneExtensions, graph_lane_y};
use crate::editors::shared::model::{EditorContext, Lane};
use crate::editors::shared::ruler::data::{Ruler, RulerTicks};
use kivact::control::{Node, NodeBuilder, node};
use kivact::interaction::{DragCoordinates, EditSession, use_drag};
use kivact::primitive::{NodeExt, NodeRefExt, part};
use kivact::style::{ColorToken, Overflow, style};
use kivact::ui::{EditPhase, ListItem};
use kivact::view::{
    A11yAction, Handled, IntoValue, IntoView, Key, MouseButton, Role, View, callback, node_ref,
};
use kivact::{Callback, Value, signal};
use std::rc::Rc;

/// An editable graph whose curves and view intervals use application domain units.
/// Curves are read-only: `on_edit` reports requests for the application to apply.
/// Drag requests are absolute from the initial keys; ignored requests do not move them.
/// Key IDs must be unique across all curves. Selection does not emit edit phases.
pub struct GraphEditor {
    root: Node,
    curves: Value<Vec<Curve>>,
    state: crate::GraphState,
    lanes: Option<Value<Vec<Lane>>>,
    snap: Value<bool>,
    settings: Settings,
    on_edit: Option<
        Callback<
            crate::editors::shared::model::EditorEvent<crate::editors::graph::model::GraphRequest>,
        >,
    >,
    extensions: LaneExtensions,
}
/// Plot application curves without converting their time or value coordinates to pixels.
///
/// `state` holds the shown time and value ranges (an invalid axis draws with
/// its default range), the selected key IDs and the playhead, in application
/// time units.
#[track_caller]
pub fn graph_editor(curves: impl IntoValue<Vec<Curve>>, state: crate::GraphState) -> GraphEditor {
    GraphEditor {
        root: node().role(Role::Canvas).focusable(true).style(
            style()
                .height(280)
                .min_width(0)
                .overflow(Overflow::Hidden)
                .background(ColorToken::Surface),
        ),
        curves: curves.into_value(),
        state,
        lanes: None,
        snap: false.into_value(),
        settings: Settings {
            ruler: Ruler {
                format: Rc::new(|v| v.to_string()),
                ..Ruler::default()
            },
            value_format: Rc::new(|v| v.to_string()),
            step: (1.0.into_value(), 0.01.into_value()),
            cells_width: 0.0,
        },
        on_edit: None,
        extensions: LaneExtensions::default(),
    }
}
impl GraphEditor {
    crate::editors::shared::lane::lane_extension_methods!(extensions);
    /// Display only curves whose IDs appear in these lanes; an explicit empty list hides all.
    pub fn lanes(mut self, lanes: impl IntoValue<Vec<Lane>>) -> Self {
        self.lanes = Some(lanes.into_value());
        self
    }
    /// Round edited positions to the corresponding time and value steps when enabled.
    pub fn snap(mut self, snap: impl IntoValue<bool>) -> Self {
        self.snap = snap.into_value();
        self
    }
    /// Set the keyboard and snapping increments of time and value. They may
    /// change at runtime; a value that is not finite and positive falls back
    /// to its default (1 and 0.01).
    pub fn step(mut self, time: impl IntoValue<f64>, value: impl IntoValue<f64>) -> Self {
        self.settings.step = (time.into_value(), value.into_value());
        self
    }
    /// Format time axis, key names and the playhead value.
    pub fn time_format(mut self, format: impl Fn(f64) -> String + 'static) -> Self {
        self.settings.ruler.format = Rc::new(format);
        self
    }
    /// Format value axis labels and key values.
    pub fn value_format(mut self, format: impl Fn(f64) -> String + 'static) -> Self {
        self.settings.value_format = Rc::new(format);
        self
    }
    /// Configure the common nice tick interval rules on both axes.
    pub fn ruler_ticks(mut self, ticks: RulerTicks) -> Self {
        self.settings.ruler.ticks = ticks;
        self
    }
    /// Report edit phases and absolute requests; the application applies item changes.
    pub fn on_edit<R: kivact::view::HandlerResult>(
        mut self,
        observer: impl Fn(
            crate::editors::shared::model::EditorEvent<crate::editors::graph::model::GraphRequest>,
        ) -> R
        + 'static,
    ) -> Self {
        self.on_edit = Some(callback(observer));
        self
    }
}
impl NodeBuilder for GraphEditor {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for GraphEditor {
    fn into_view(self) -> View {
        let Self {
            root,
            curves,
            state,
            lanes,
            snap,
            mut settings,
            on_edit,
            extensions,
        } = self;
        settings.cells_width = extensions
            .cells
            .iter()
            .map(|(width, _)| *width)
            .sum::<f32>()
            .min(GRAPH_LANE_WIDTH);
        settings.ruler.report();
        let settings = Rc::new(settings);
        let crate::GraphState {
            view,
            selection: selected,
            playhead,
        } = state;
        let lanes = lanes.unwrap_or_else(|| {
            kivact::memo(move || {
                curves
                    .get()
                    .iter()
                    .map(|c| Lane::new(c.id, c.label.clone()))
                    .collect()
            })
        });
        let visible = crate::editors::graph::a11y::visible(curves, lanes, settings.clone());
        let order = kivact::memo(move || {
            visible
                .get()
                .into_iter()
                .map(|k| ListItem::new(k.key.id, k.name))
                .collect::<Vec<_>>()
        });
        let choose = crate::editors::shared::selection::controller(order, selected);
        let scene = crate::editors::graph::a11y::scene(curves, visible, selected, lanes);
        let (cells, child_focus) = crate::editors::shared::lane::lane_cells(
            lanes,
            &extensions,
            |index| {
                kivact::draw::Rect::new(
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
            on_edit.map(|observer| {
                callback(move |phase| {
                    observer.call(match phase {
                        EditPhase::Begin => crate::editors::shared::model::EditorEvent::Begin,
                        EditPhase::Commit => crate::editors::shared::model::EditorEvent::Commit,
                        EditPhase::Cancel => crate::editors::shared::model::EditorEvent::Cancel,
                    })
                })
            }),
        );
        let press = crate::editors::shared::gesture::CanvasGesture::<
            crate::editors::graph::gesture::Target,
            (
                Option<crate::editors::graph::gesture::Gesture>,
                Option<crate::editors::graph::gesture::Snapshot>,
            ),
        >::new();
        // A gesture begins its edit at its first change, from the press's
        // snapshot; an empty one would restore nothing.
        let gesture = use_drag::<()>(DragCoordinates::Local)
            .buttons([MouseButton::Middle])
            .edit_on_change(edit, move || {
                press
                    .with_payload(|payload| payload.1.clone())
                    .flatten()
                    .unwrap_or_else(|| crate::editors::graph::gesture::Snapshot::Keys(Vec::new()))
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
            size: kivact::memo(move || {
                let r = rect.get().unwrap_or_default();
                kivact::draw::Size::new(r.width, r.height)
            }),
            snap,
            step: settings.step,
            edit,
            press,
            gesture,
            active: signal(None),
            space: signal(false),
            lane_activate: extensions.lane_activate,
            item_activate: extensions.item_activate,
            child_focus,
            context: signal(EditorContext {
                item: None,
                lane: None,
                time: playhead.get_untracked(),
            }),
        };
        let gesture = state.drag();
        let focus = reference.focus_within();
        kivact::effect(move || {
            let focused = focus.get();
            let visible = reference.is_visible();
            let keys = state.visible.get();
            let gesture = state.drag_state();
            if let Some(gesture) = gesture {
                let missing = match gesture.kind {
                    DragKind::Keys(ids) => {
                        ids.iter().any(|id| !keys.iter().any(|k| k.key.id == *id))
                    }
                    DragKind::Tangent(id, _) => !keys.iter().any(|k| k.key.id == id),
                    _ => false,
                };
                if !focused || !visible || missing {
                    state.cancel();
                }
            }
        });
        let root = crate::editors::graph::a11y::attach(
            root.value_text(move || {
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
            .part(gesture.source(()))
            // After the drag's sources, which take Escape during a drag.
            .part(
                part(|input| input)
                    .region_events()
                    .on_key(move |event| state.key(event)),
            )
            .on_key_up(move |event| {
                if event.key == Key::Space {
                    state.space.set(false);
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
            .on_wheel(move |event| state.wheel(event));
        match extensions.menu {
            Some(menu) => menu(
                root.on_a11y_event(A11yAction::ShowMenu, move |event| {
                    state.cancel();
                    state.context_request(event.region);
                }),
                state.context.into_value(),
            ),
            None => root.into_view(),
        }
    }
}
