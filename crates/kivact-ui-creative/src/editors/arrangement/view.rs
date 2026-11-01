//! Role: The Arrangement builder and the track canvas view it builds.

use super::gesture::State;
use crate::ArrangementState;
use crate::Clip;
use crate::editors::arrangement::a11y::texts::ArrangementTexts;
use crate::editors::arrangement::model::{ArrangementRequest, TimelineMarker, TimelineTrack};
use kivact::control::{Node, NodeBuilder, node};
use kivact::interaction::{DragCoordinates, EditSession};
use kivact::primitive::{NodeExt, NodeRefExt};
use kivact::style::{Cursor, Overflow, pct, style};
use kivact::ui::EditPhase;
use kivact::view::MouseButton;
use kivact::view::{
    A11yAction, HandlerResult, IntoValue, IntoView, Role, View, callback, node_ref,
};
use kivact::{Callback, Value, signal};

/// A complete arrange surface with loop locators, markers, tracks, clips and automation.
pub struct Arrangement {
    root: Node,
    items: Value<Vec<Clip>>,
    tracks: Value<Vec<TimelineTrack>>,
    length: Value<f64>,
    ruler: crate::editors::shared::ruler::Ruler,
    loop_min_duration: f64,
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
            zoom: (8.0, 80.0),
            min_duration: 0.0625,
            ..Default::default()
        },
        loop_min_duration: crate::editors::shared::metrics::LOOP_MIN_LENGTH,
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
    pub fn zoom_range(mut self, min: f64, max: f64) -> Self {
        if min.is_finite() && max.is_finite() && min > 0.0 && min <= max {
            self.ruler.zoom = (min, max);
        } else {
            self.ruler
                .usage
                .push("arrangement zoom range must be finite, positive and ordered".into());
        }
        self
    }
    /// Smallest proposed clip duration in beats; defaults to 1/16.
    #[track_caller]
    pub fn min_duration(mut self, beats: f64) -> Self {
        if beats.is_finite() && beats > 0.0 {
            self.ruler.min_duration = beats;
        } else {
            self.ruler
                .usage
                .push("arrangement minimum duration must be finite and positive".into());
        }
        self
    }
    /// Smallest proposed loop range in beats; defaults to 1/4.
    #[track_caller]
    pub fn loop_min_duration(mut self, beats: f64) -> Self {
        if beats.is_finite() && beats > 0.0 {
            self.loop_min_duration = beats;
        } else {
            self.ruler
                .usage
                .push("arrangement loop minimum duration must be finite and positive".into());
        }
        self
    }
    /// Run when the add-track action is invoked. Setup only.
    #[track_caller]
    pub fn on_add_track<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.add_track = Some(callback(move |_| f()));
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
        let length = kivact::memo(move || {
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
        let order = kivact::memo(move || {
            data.get()
                .into_iter()
                .filter(crate::editors::shared::model::drawable)
                .map(|c| kivact::ui::ListItem::new(c.id, c.label))
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
        let press = crate::editors::shared::gesture::CanvasGesture::<
            crate::editors::arrangement::a11y::targets::Target,
            (
                Option<crate::editors::arrangement::gesture::Drag>,
                Option<crate::editors::arrangement::gesture::Snapshot>,
            ),
        >::new();
        // A gesture begins its edit at its first change, from the press's
        // snapshot; an empty one would restore nothing.
        let gesture = kivact::interaction::use_drag::<()>(DragCoordinates::Local)
            .buttons([MouseButton::Middle])
            .edit_on_change(edit, move || {
                press
                    .with_payload(|payload| payload.1.clone())
                    .flatten()
                    .unwrap_or_else(|| super::gesture::Snapshot::Clips(Vec::new()))
            });
        let state = State {
            size: kivact::memo(move || {
                let r = rect.get().unwrap_or_default();
                kivact::draw::Size::new(r.width, r.height)
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
        let focus = reference.focus_within();
        kivact::effect(move || {
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
            if state.drag_state().is_some() && (!focused || !visible || missing) {
                state.cancel();
            }
        });
        let action = callback(move |event| super::edit::a11y(state, add_track, event));
        let semantic_items = kivact::memo(move || super::a11y::semantic(state));
        let root = root.regions(
            semantic_items,
            |info| info.id,
            move |_, info| {
                // A press on a region drags from there.
                crate::editors::shared::region_parts::attachment(
                    info.get_untracked().kind,
                    info,
                    action,
                )
                .with(gesture.source(()))
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
                let t = &kivact::style::current_theme();
                style().background(t.get::<crate::tokens::Tokens>().island)
            })
            .on_a11y_event(A11yAction::Select, move |event| action.call(event))
            .on_a11y_event(A11yAction::Activate, move |event| action.call(event))
            .on_a11y_event(A11yAction::SetValue, move |event| action.call(event))
            .draw(move |d, size| super::paint::paint(d, size, state))
            .on_pointer(move |e| super::input::pointer(state, e))
            .on_wheel(move |e| super::input::wheel(state, e))
            .part(gesture.source(()))
            // After the drag's sources, which take Escape during a drag.
            .part(
                kivact::primitive::part(|input| input)
                    .region_events()
                    .on_key(move |e| super::input::key(state, e)),
            )
            .into_view()
    }
}
