//! Role: The Arrangement builder and the track canvas view it builds.

use super::gesture::State;
use crate::ArrangementState;
use crate::Clip;
use crate::editors::arrangement::a11y::texts::ArrangementTexts;
use crate::editors::arrangement::model::{ArrangementRequest, TimelineMarker, TimelineTrack};
use kivact::custom::NodeRefExt;

use kivact::interaction::{DragCoordinates, EditSession};
use kivact::state::{Callback, Value, signal};
use kivact::state::{IntoValue, callback};
use kivact::style::{Cursor, Overflow, pct, style};
use kivact::view::EditPhase;
use kivact::view::MouseButton;
use kivact::view::{A11yAction, HandlerResult, IntoView, Role, View, node_ref};
use kivact::view::{Node, NodeBuilder, node};

/// A complete arrange surface with loop locators, markers, tracks, clips and automation.
pub struct Arrangement {
    root: Node,
    items: Value<Vec<Clip>>,
    tracks: Value<Vec<TimelineTrack>>,
    length: Value<f64>,
    ruler: crate::editors::shared::ruler::Ruler,
    loop_min_duration: Value<f64>,
    state: ArrangementState,
    markers: Value<Vec<TimelineMarker>>,
    snap: Value<bool>,
    step: Value<f64>,
    add_track: Option<Callback<()>>,
    on_edit: Option<Callback<crate::editors::shared::model::EditorEvent<ArrangementRequest>>>,
}

/// Clip positions, widths and `length` use beats; the zoom of `state` is
/// pixels per beat. Item lists are read-only. Apply `on_edit` requests in the
/// application. `state` holds the selection, the playhead, the loop, the
/// zoom, the scroll and the tool, which the arrangement writes and reads.
/// A non-finite or non-positive length reports a UI error and uses an empty range.
#[track_caller]
pub fn arrangement(
    clips: impl IntoValue<Vec<Clip>>,
    tracks: impl IntoValue<Vec<TimelineTrack>>,
    state: ArrangementState,
) -> Arrangement {
    Arrangement {
        root: node().role(Role::Canvas).focusable(true).style(
            style()
                .width(pct(100))
                .height(380)
                .min_width(0)
                .overflow(Overflow::Hidden)
                .cursor(Cursor::Default),
        ),
        items: clips.into_value(),
        tracks: tracks.into_value(),
        length: 16_f64.into_value(),
        ruler: crate::editors::shared::ruler::Ruler {
            zoom: kivact::interaction::ScalarRange::new(8.0, 80.0)
                .expect("invariant: default arrangement zoom")
                .into_value(),
            min_duration: 0.0625.into_value(),
            ..Default::default()
        },
        loop_min_duration: crate::editors::shared::metrics::LOOP_MIN_LENGTH.into_value(),
        state,
        markers: (move || Vec::new()).into_value(),
        snap: true.into_value(),
        step: 1.0.into_value(),
        add_track: None,
        on_edit: None,
    }
}

impl Arrangement {
    /// Reactive extent in beats; sixteen by default.
    pub fn length(mut self, length: impl IntoValue<f64>) -> Self {
        self.length = length.into_value();
        self
    }

    /// Receive absolute item proposals and gesture boundaries.
    /// The application applies requests and restores its items on Cancel.
    pub fn on_edit<R: HandlerResult>(
        mut self,
        f: impl Fn(crate::editors::shared::model::EditorEvent<ArrangementRequest>) -> R + 'static,
    ) -> Self {
        self.on_edit = Some(callback(f));
        self
    }
    /// Logical pixels per beat; the default range is 8..80.
    #[track_caller]
    pub fn zoom_range(
        mut self,
        range: impl kivact::state::IntoValue<kivact::interaction::ScalarRange>,
    ) -> Self {
        self.ruler.zoom = kivact::state::IntoValue::into_value(range);
        self
    }
    /// Smallest proposed clip duration in beats; defaults to 1/16.
    #[track_caller]
    pub fn min_duration(mut self, duration: impl kivact::state::IntoValue<f64>) -> Self {
        self.ruler.min_duration = crate::editors::shared::ruler::duration(duration, 0.0625);
        self
    }
    /// Smallest proposed loop range in beats; defaults to 1/4.
    #[track_caller]
    pub fn loop_min_duration(mut self, duration: impl kivact::state::IntoValue<f64>) -> Self {
        self.loop_min_duration = crate::editors::shared::ruler::duration(duration, 0.0625);
        self
    }
    /// Run when the add-track action is invoked. Setup only.
    #[track_caller]
    pub fn on_add_track(mut self, f: impl Into<kivact::state::Callback<()>>) -> Self {
        self.add_track = Some(f.into());
        self
    }
    pub fn markers(mut self, markers: impl IntoValue<Vec<TimelineMarker>>) -> Self {
        self.markers = markers.into_value();
        self
    }
    pub fn snap(mut self, snap: impl IntoValue<bool>) -> Self {
        self.snap = snap.into_value();
        self
    }
    pub fn step(mut self, beats: impl IntoValue<f64>) -> Self {
        self.step = beats.into_value();
        self
    }
}

impl NodeBuilder for Arrangement {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Arrangement {
    fn into_view(self) -> View {
        let Self {
            root,
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
                    zoom,
                    pan,
                    tool: tool_binding,
                },
            markers,
            snap,
            step,
            add_track,
            on_edit,
        } = self;
        ruler.report();
        let report = callback(|error| -> kivact::Result<()> { Err(error) });
        let length = kivact::state::memo(move || {
            let value = length.get();
            if value.is_finite() && value > 0.0 {
                value
            } else {
                report.call(kivact::UiError::msg(
                    "arrangement length must be finite and positive",
                ));
                0.0
            }
        });
        let texts = ArrangementTexts::new();
        let data = items.into_value();
        let track_data = tracks.into_value();

        let selection = selected.into_value();
        let regions =
            crate::editors::arrangement::a11y::targets::create(track_data, data, selection);
        let order = kivact::state::memo(move || {
            data.get()
                .into_iter()
                .filter(crate::editors::shared::model::drawable)
                .map(|c| kivact::interaction::ListItem::new(c.id, c.label))
                .collect::<Vec<_>>()
        });
        let choose = crate::editors::shared::selection::controller(order, selected);
        let frame = playhead.into_value();
        let range = loop_range.into_value();
        let cycle = cycling.into_value();
        let pixels = zoom.into_value();
        let offset = pan.into_value();
        let tool = tool_binding.into_value();
        let active_auto = signal(None::<(u64, u64)>);
        let restore = super::edit::Restore {
            playhead,
            loop_range,
            cycling,
            zoom,
            pan,
        };
        let reference = node_ref();
        let rect = reference.rect();
        let edit = crate::editors::shared::model::observed(
            EditSession::new(move |snapshot| super::edit::restore(restore, snapshot)),
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
        let gesture =
            kivact::interaction::use_drag::<super::gesture::Snapshot>(DragCoordinates::Local)
                .slop(0.0)
                .buttons([MouseButton::Middle]);
        let press = crate::editors::shared::gesture::CanvasGesture::new(gesture);
        let gesture = gesture.edit_on_change(edit, move || {
            press
                .initial()
                .unwrap_or_else(|| super::gesture::Snapshot::Clips(Vec::new()))
        });
        let state = State {
            size: kivact::state::memo(move || {
                let r = rect.get().unwrap_or_default();
                kivact::geometry::Size::new(r.width, r.height)
            }),
            data,
            length,
            bounds: ruler.zoom,
            min_duration: ruler.min_duration,
            loop_min_duration,
            on_edit,
            track_data,
            selected,
            selection,
            playhead,
            frame,
            loop_range,
            range,
            cycle,
            cycling,
            zoom,
            pixels,
            pan,
            offset,
            tool_binding,
            tool,
            markers,
            snap,
            step,
            active_auto,
            edit,
            press,
            gesture,
            regions,
            texts,
            choose,
        };
        let gesture = super::input::drag(state);
        let availability = kivact::custom::part(|input| input);
        let availability_state = availability.state();
        let focus = reference.focus_within();
        kivact::state::effect(move || {
            let focused = focus.get();
            let visible = reference.is_visible();
            let clips = state.data.get();
            let tracks = state.track_data.get();
            let missing = state.initial().is_some_and(|v| match v {
                super::gesture::Snapshot::Clips(initial) => initial.iter().any(|v| {
                    !clips
                        .iter()
                        .any(|c| c.id == v.0 && tracks.iter().any(|t| t.id == c.lane))
                }),
                super::gesture::Snapshot::Automation(track, id, Some(_)) => !tracks
                    .iter()
                    .any(|t| t.id == track && t.automation.iter().any(|p| p.id == id)),
                super::gesture::Snapshot::Automation(track, _, None) => {
                    !tracks.iter().any(|t| t.id == track)
                }
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
            .style(move || {
                let t = &kivact::style::use_theme().get();
                style().background(t.ext::<crate::tokens::Tokens>().island)
            })
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
            .into_view()
    }
}
