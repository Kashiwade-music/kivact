//! Role: Shared time-item geometry, drawing and gestures for clips and keys.
pub(crate) mod a11y;
pub(crate) mod edit;
pub(crate) mod gesture;
pub(crate) mod input;
pub(crate) mod layout;
pub(crate) mod model;
pub(crate) mod paint;
pub(crate) mod view_state;

use crate::editors::shared::lane::LaneExtensions;
use crate::editors::shared::metrics::SIDEBAR;
use crate::editors::shared::model::{EditorContext, Lane};
use crate::editors::shared::ruler::data::Ruler;
use crate::editors::shared::selection;
use crate::editors::shared::targets;
use crate::editors::time::gesture::{State, Target};
use crate::editors::time::input::{key, pointer, wheel};
use crate::editors::time::layout::lane_rect;
use crate::editors::time::model::TimeItem;
use kivact::control::{Node, node};
use kivact::interaction::{DragCoordinates, EditSession, use_drag};
use kivact::prelude::NodeBuilder;
use kivact::primitive::{NodeExt, NodeRefExt, part};
use kivact::style::{ColorToken, Overflow, style};
use kivact::ui::{EditPhase, ListItem};
use kivact::view::{A11yAction, IntoValue, IntoView, MouseButton, Role, View, callback, node_ref};
use kivact::{Callback, Value};

pub(crate) struct TimeEditor<T: TimeItem> {
    pub(crate) root: Node,
    pub(crate) items: Value<Vec<T>>,
    pub(crate) state: crate::TimeState,
    pub(crate) lanes: Option<Value<Vec<Lane>>>,
    pub(crate) snap: Value<bool>,
    pub(crate) step: Value<f64>,
    pub(crate) ruler: Ruler,
    pub(crate) lane_height: f32,
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
        lane_height: f32,
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
        let Self {
            root,
            items,
            state:
                crate::TimeState {
                    selection: selected,
                    playhead,
                    zoom,
                    pan,
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
        let data = kivact::memo(move || {
            source
                .get()
                .into_iter()
                .filter(TimeItem::valid)
                .collect::<Vec<_>>()
        });
        let scale = zoom.into_value();
        let offset = pan.into_value();
        let _frame = playhead.into_value();
        let visible = kivact::memo(move || {
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
        });
        let order = kivact::memo(move || {
            data.get()
                .into_iter()
                .filter(|item| {
                    visible.with(|lanes| lanes.iter().any(|lane| lane.id == item.lane()))
                })
                .map(|item| ListItem::new(item.id(), item.label()))
                .collect::<Vec<_>>()
        });
        let choose = selection::controller(order, selected);
        let scene = targets::targets(move || {
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
        });
        let cells_width = extensions
            .cells
            .iter()
            .map(|(width, _)| *width)
            .sum::<f32>()
            .min(SIDEBAR);
        let (cells, child_focus) = crate::editors::shared::lane::lane_cells(
            visible,
            &extensions,
            move |index| lane_rect(index, lane_height),
            true,
        );
        let edit = crate::editors::shared::model::observed(
            EditSession::new(move |snapshot| {
                crate::editors::time::edit::restore(playhead, zoom, pan, snapshot)
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
            crate::editors::time::gesture::Target,
            (
                Option<crate::editors::time::gesture::Drag>,
                Option<crate::editors::time::gesture::Snapshot>,
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
                    .unwrap_or_else(|| crate::editors::time::gesture::Snapshot::Items(Vec::new()))
            });
        let state = State {
            data,
            on_edit,
            selected,
            selection,
            zoom,
            scale,
            pan,
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
            active: kivact::signal(None),
            child_focus,
            context: kivact::signal(EditorContext {
                item: None,
                lane: None,
                time: playhead.get_untracked(),
            }),
            lane_activate: extensions.lane_activate,
            item_activate: extensions.item_activate,
            press,
            gesture,
            edit,
        };
        let gesture = crate::editors::time::input::drag(state);
        let reference = node_ref();
        let focus = reference.focus_within();
        kivact::effect(move || {
            let focused = focus.get();
            let visible = reference.is_visible();
            let items = state.data.get();
            let lanes = state.visible.get();
            let missing = state.initial().is_some_and(|snapshot| match snapshot {
                crate::editors::time::gesture::Snapshot::Items(initial) => {
                    initial.iter().any(|v| {
                        !items
                            .iter()
                            .any(|i| i.id() == v.0 && lanes.iter().any(|lane| lane.id == i.lane()))
                    })
                }
                _ => false,
            });
            if state.drag_state().is_some() && (!focused || !visible || missing) {
                state.cancel();
            }
        });
        let root = a11y::attach(root, state, gesture, ruler.format.clone());
        let root = root
            .node_ref(reference)
            .child(cells)
            .value_text(move || {
                format!(
                    "{} items; {} selected; zoom {:.0}%",
                    data.with(Vec::len),
                    selection.with(Vec::len),
                    scale.get().clamp(bounds.0, bounds.1) * 100.0
                )
            })
            .draw(move |draw, size| paint::paint(draw, size, state, &ruler, cells_width))
            .on_pointer(move |event| state.context_pointer(event))
            .on_key(move |event| state.context_key(&event))
            .on_pointer_default(move |event| pointer(state, event))
            .on_wheel_default(move |event| wheel(state, event))
            .part(gesture.source(()))
            // After the drag's sources, which take Escape during a drag.
            .part(
                part(|input| input)
                    .region_events()
                    .on_key(move |event| key(state, event)),
            );
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

macro_rules! time_editor_builder {
    ($type:ident, $function:ident, $item:path, $height:expr, $type_doc:literal, $function_doc:literal) => {
        #[doc = $type_doc]
        pub struct $type {
            inner: $crate::editors::time::TimeEditor<$item>,
        }
        #[doc = $function_doc]
        #[track_caller]
        pub fn $function(
            items: impl kivact::view::IntoValue<Vec<$item>>,
            state: $crate::TimeState,
        ) -> $type {
            $type {
                inner: $crate::editors::time::TimeEditor::new(items, state, $height),
            }
        }
        impl $type {
            $crate::editors::shared::lane::lane_extension_methods!(inner.extensions);
            /// Report absolute item proposals and gesture boundaries.
            /// The application applies requests and restores its items on cancellation.
            pub fn on_edit<R: kivact::view::HandlerResult>(
                mut self,
                f: impl Fn(
                    $crate::editors::shared::model::EditorEvent<
                        $crate::editors::time::model::TimeRequest,
                    >,
                ) -> R
                + 'static,
            ) -> Self {
                self.inner.on_edit = Some(kivact::view::callback(f));
                self
            }
            /// Provide reactive lane labels and ordering.
            pub fn lanes(
                mut self,
                lanes: impl kivact::view::IntoValue<Vec<$crate::editors::shared::model::Lane>>,
            ) -> Self {
                self.inner.lanes = Some(kivact::view::IntoValue::into_value(lanes));
                self
            }
            /// Enable snapping for pointer edits.
            pub fn snap(mut self, snap: impl kivact::view::IntoValue<bool>) -> Self {
                self.inner.snap = kivact::view::IntoValue::into_value(snap);
                self
            }
            /// Set the reactive time increment for keys and pointer snapping (default: 1).
            /// Non-finite or non-positive values use 1. Shift keys move by ten increments.
            /// Moves beyond the finite numeric range leave the value unchanged.
            pub fn step(mut self, step: impl kivact::view::IntoValue<f64>) -> Self {
                self.inner.step = kivact::view::IntoValue::into_value(step);
                self
            }
            /// Format ruler ticks and the playhead using application time units.
            pub fn time_format(mut self, format: impl Fn(f64) -> String + 'static) -> Self {
                self.inner.ruler.format = std::rc::Rc::new(format);
                self
            }
            /// Choose automatic or application-provided ruler ticks.
            pub fn ruler_ticks(
                mut self,
                ticks: $crate::editors::shared::ruler::data::RulerTicks,
            ) -> Self {
                self.inner.ruler.ticks = ticks;
                self
            }
            /// Set finite positive zoom bounds; invalid bounds report a usage error.
            pub fn zoom_range(mut self, min: f64, max: f64) -> Self {
                if min.is_finite() && max.is_finite() && min > 0.0 && min <= max {
                    self.inner.ruler.zoom = (min, max);
                } else {
                    self.inner.ruler.usage.push(format!(
                        "{}::zoom_range({min}, {max}) needs finite bounds with 0 < min <= max",
                        stringify!($type),
                        min = min,
                        max = max
                    ));
                }
                self
            }
            /// Set the minimum clip duration; invalid lengths report a usage error.
            pub fn min_duration(mut self, duration: f64) -> Self {
                if duration.is_finite() && duration >= 0.0 {
                    self.inner.ruler.min_duration = duration;
                } else {
                    self.inner.ruler.usage.push(format!(
                        "{}::min_duration({duration}) needs a finite, non-negative length",
                        stringify!($type),
                        duration = duration
                    ));
                }
                self
            }
        }
        impl kivact::control::NodeBuilder for $type {
            /// Return the editor root for ordinary node modifiers.
            fn node_mut(&mut self) -> &mut kivact::control::Node {
                &mut self.inner.root
            }
        }
        #[diagnostic::do_not_recommend]
        impl kivact::view::IntoView for $type {
            /// Build the editor view.
            fn into_view(self) -> kivact::view::View {
                kivact::view::IntoView::into_view(self.inner)
            }
        }
    };
}
pub(crate) use time_editor_builder;
