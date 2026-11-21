//! Role: The Arrangement component and the track canvas view it builds.

use super::gesture::State;
use crate::ArrangementState;
use crate::Clip;
use crate::editors::arrangement::a11y::texts::ArrangementTexts;
use crate::editors::arrangement::model::{ArrangementRequest, TimelineMarker, TimelineTrack};
use crate::editors::shared::metrics::LOOP_MIN_LENGTH;
use crate::editors::shared::model::EditorEvent;
use crate::editors::shared::ruler::{Ruler, duration};

use kivact::component;
use kivact::interaction::{EditSession, ScalarRange};
use kivact::state::{Callback, IntoValue, Value, callback};
use kivact::style::{Cursor, Overflow, pct, style};
use kivact::view::{A11yAction, Node, Role, node, node_ref};

/// A complete arrange surface with loop locators, markers, tracks, clips and automation.
///
/// Clip positions, widths and `length` use beats; the zoom of `state` is
/// pixels per beat. Item lists are read-only. Apply `on_edit` requests in the
/// application. `state` holds the selection, the playhead, the loop, the
/// zoom, the scroll and the tool, which the arrangement writes and reads.
/// A non-finite or non-positive length reports a UI error and uses an empty range.
#[component]
pub fn arrangement(
    clips: Value<Vec<Clip>>,
    tracks: Value<Vec<TimelineTrack>>,
    state: ArrangementState,
    #[prop(default = 16.0)] length: Value<f64>,
    zoom_range: Option<Value<ScalarRange>>,
    min_duration: Option<Value<f64>>,
    loop_min_duration: Option<Value<f64>>,
    on_add_track: Option<Callback<()>>,
    on_edit: Option<Callback<EditorEvent<ArrangementRequest>>>,
    #[prop(default = Vec::new())] markers: Value<Vec<TimelineMarker>>,
    #[prop(default = true)] snap: Value<bool>,
    #[prop(default = 1.0)] step: Value<f64>,
) -> impl Element {
    let zoom = match zoom_range {
        Some(range) => range,
        None => ScalarRange::new(8.0, 80.0)
            .expect("invariant: default arrangement zoom")
            .into_value(),
    };
    let min_duration = match min_duration {
        Some(value) => duration(value, 0.0625),
        None => 0.0625.into_value(),
    };
    let loop_min_duration = match loop_min_duration {
        Some(value) => duration(value, 0.0625),
        None => LOOP_MIN_LENGTH.into_value(),
    };
    let ruler = Ruler {
        zoom,
        min_duration,
        ..Default::default()
    };
    ArrangementView {
        items: clips,
        tracks,
        length,
        ruler,
        loop_min_duration,
        state,
        markers,
        snap,
        step,
        add_track: on_add_track,
        on_edit,
    }
    .into_node()
}

/// The settings a component collects, which build the canvas of the arrangement.
struct ArrangementView {
    items: Value<Vec<Clip>>,
    tracks: Value<Vec<TimelineTrack>>,
    length: Value<f64>,
    ruler: Ruler,
    loop_min_duration: Value<f64>,
    state: ArrangementState,
    markers: Value<Vec<TimelineMarker>>,
    snap: Value<bool>,
    step: Value<f64>,
    add_track: Option<Callback<()>>,
    on_edit: Option<Callback<EditorEvent<ArrangementRequest>>>,
}

impl ArrangementView {
    fn into_node(self) -> Node {
        let Prepared {
            root,
            state,
            reference,
            add_track,
        } = self.prepare();
        let data = state.data;
        let track_data = state.track_data;
        let selection = state.selection;
        let gesture = super::input::drag(state);
        let availability = kivact::custom::part(|input| input);
        state.press.watch_inputs(
            crate::editors::shared::gesture::inputs_ok(reference, availability.state()),
            move || state.cancel(),
        );
        let action = callback(move |event| super::edit::a11y(state, add_track, event));
        let semantic_items = kivact::state::memo(move || super::a11y::semantic(state));
        let root = root.regions(
            semantic_items,
            |info| info.id,
            move |_, info| {
                // A press on a region drags from there.
                let attachment = crate::editors::shared::region_parts::attachment(
                    info.get_untracked().kind,
                    info,
                    action,
                );
                let drag_source = gesture.source_with(move || super::input::source_snapshot(state));
                kivact::custom::Part::new(attachment.state(), move |node| {
                    node.part(attachment).part(drag_source)
                })
            },
        );
        root.node_ref(reference)
            .value_text(move || {
                format!(
                    "{} clips; {} tracks; {} selected",
                    data.with(Vec::len),
                    track_data.with(Vec::len),
                    selection.with(Vec::len)
                )
            })
            .style(style().background(crate::color::ISLAND))
            .a11y_action(A11yAction::Select, move |event| action.call(event))
            .a11y_action(A11yAction::Activate, move |event| action.call(event))
            .a11y_action(A11yAction::SetValue, move |event| action.call(event))
            .draw(move |d, size| super::paint::paint(d, size, state))
            .on_pointer(move |e| super::input::pointer(state, e))
            .on_wheel(move |e| super::input::wheel(state, e))
            .part(availability)
            .part(gesture.source_with(move || super::input::source_snapshot(state)))
            // After the drag's sources, which take Escape during a drag.
            .part({
                let input_part = kivact::custom::part(|input| input);
                kivact::custom::Part::new(input_part.state(), move |node| {
                    node.part(input_part)
                        .region_events()
                        .on_key(move |e| super::input::key(state, e))
                })
            })
    }
}

struct Prepared {
    root: Node,
    state: State,
    reference: kivact::view::NodeRef,
    add_track: Option<Callback<()>>,
}
impl ArrangementView {
    fn prepare(self) -> Prepared {
        let root = canvas_root();
        let Self {
            items,
            tracks,
            length,
            ruler,
            loop_min_duration,
            state:
                ArrangementState {
                    selection: selected,
                    playhead,
                    loop_range,
                    cycling,
                    transform,
                    tool: tool_binding,
                },
            markers,
            snap,
            step,
            add_track,
            on_edit,
        } = self;
        ruler.report();
        let length = validated_length(length);
        let texts = ArrangementTexts::new();
        let data = items.into_value();
        let track_data = tracks.into_value();

        let collection = collection_inputs(data, track_data, selected);
        let frame = playhead.into_value();
        let range = loop_range.into_value();
        let cycle = cycling.into_value();
        let pixels = kivact::state::memo(move || transform.get().scale);
        let offset = kivact::state::memo(move || transform.get().offset);
        let tool = tool_binding.into_value();
        let reference = node_ref();
        let rect = reference.rect();
        let edit = arrangement_edit(playhead, loop_range, cycling, transform, on_edit);
        let (press, gesture) = crate::editors::shared::gesture::CanvasGesture::editor(edit, || {
            super::gesture::Snapshot::Clips(Vec::new())
        });
        let state = State {
            size: crate::editors::shared::model::canvas_size(rect),
            data,
            length,
            bounds: ruler.zoom,
            min_duration: ruler.min_duration,
            loop_min_duration,
            on_edit,
            track_data,
            selected,
            selection: collection.selection,
            playhead,
            frame,
            loop_range,
            range,
            cycle,
            cycling,
            transform,
            pixels,
            offset,
            tool_binding,
            tool,
            markers,
            snap,
            step,
            active_auto: collection.active_auto,
            edit,
            press,
            gesture,
            regions: collection.regions,
            texts,
            choose: collection.choose,
        };
        Prepared {
            root,
            state,
            reference,
            add_track,
        }
    }
}

/// The focusable canvas that the regions, the pointer and the keys attach to.
fn canvas_root() -> Node {
    node().role(Role::Canvas).focusable(true).style(
        style()
            .width(pct(100))
            .height(380)
            .min_width(0)
            .overflow(Overflow::Hidden)
            .cursor(Cursor::Default),
    )
}

fn validated_length(length: Value<f64>) -> Value<f64> {
    let report = callback(|error| -> kivact::Result<()> { Err(error) });
    kivact::state::memo(move || {
        let value = length.get();
        if value.is_finite() && value > 0.0 {
            value
        } else {
            report.call(kivact::UiError::msg(
                "arrangement length must be finite and positive",
            ));
            0.0
        }
    })
    .into_value()
}

struct CollectionInputs {
    selection: Value<Vec<u64>>,
    regions: Value<crate::editors::arrangement::a11y::targets::Regions>,
    active_auto: kivact::state::Signal<Option<(u64, u64)>>,
    choose: kivact::interaction::ListApi<u64>,
}
fn collection_inputs(
    data: Value<Vec<Clip>>,
    track_data: Value<Vec<TimelineTrack>>,
    selected: kivact::state::Signal<Vec<u64>>,
) -> CollectionInputs {
    let selection = selected.into_value();
    let (regions, active_auto) =
        crate::editors::arrangement::a11y::targets::create(track_data, data, selection);
    let order = kivact::state::memo(move || {
        data.get()
            .into_iter()
            .filter(crate::editors::shared::model::drawable)
            .map(|c| kivact::interaction::ListItem::new(c.id, c.label))
            .collect::<Vec<_>>()
    });
    let choose = crate::editors::shared::selection::controller(order, selected);
    CollectionInputs {
        selection,
        regions,
        active_auto,
        choose,
    }
}

fn arrangement_edit(
    playhead: kivact::state::Signal<f64>,
    loop_range: kivact::state::Signal<(f64, f64)>,
    cycling: kivact::state::Signal<bool>,
    transform: kivact::state::Signal<kivact::interaction::ViewTransform>,
    on_edit: Option<Callback<EditorEvent<ArrangementRequest>>>,
) -> EditSession<super::gesture::Snapshot> {
    let restore = super::edit::Restore {
        playhead,
        loop_range,
        cycling,
        transform,
    };
    crate::editors::shared::model::observed(
        EditSession::new(move |snapshot| super::edit::restore(restore, snapshot)),
        crate::editors::shared::model::phase_observer(on_edit),
    )
}
