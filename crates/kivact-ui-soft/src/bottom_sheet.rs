//! Role: Present a bottom-aligned sheet with draggable detents.
use crate::color as c;
use crate::recipe_parts::{body_text, shadowed, tween};
use crate::shadows;

use kivact::component;
use kivact::interaction::DialogMode;
use kivact::interaction::DismissOn;
use kivact::interaction::DragAxis;
use kivact::interaction::DragCoordinates;
use kivact::interaction::DragPhase;
use kivact::interaction::DragUpdate;
use kivact::interaction::nearest_snap;
use kivact::interaction::project_release;
use kivact::interaction::pull_to_snap;
use kivact::interaction::resist_overshoot;
use kivact::interaction::use_dialog;
use kivact::interaction::use_drag;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Machine;
use kivact::state::MachineHandle;
use kivact::state::PropValue;
use kivact::state::Signal;
use kivact::state::Step;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::machine;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::Overflow;
use kivact::style::Position;
use kivact::style::ThemeMode;
use kivact::style::pct;
use kivact::style::style;
use kivact::style::use_theme;
use kivact::view::DismissReason;
use kivact::view::Element;
use kivact::view::NodeRef;
use kivact::view::Placement;
use kivact::view::Role;
use kivact::view::ViewFn;
use kivact::view::column;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::overlay;
use kivact::view::scroll;
use kivact::view::show;
use std::rc::Rc;
use std::time::Duration;

const PROJECTION_MIN_TRAVEL: f64 = 48.0;
const PROJECTION_DECAY: Duration = Duration::from_millis(499);
const SHEET_AXIS: DragAxis = DragAxis::Vertical;
const DISMISS_FRACTION: f64 = 0.4;
const OVERSHOOT_RESISTANCE: f64 = 0.35;
const OVERSHOOT_LIMIT: f64 = 48.0;
const SNAP_PULL_RADIUS: f64 = 40.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Sheet {
    Resting { snap: usize },
    Dragging { from: usize, offset: f64 },
    Settling { to: usize },
}

impl Sheet {
    fn snap(self) -> usize {
        match self {
            Self::Resting { snap } => snap,
            Self::Dragging { from, .. } => from,
            Self::Settling { to } => to,
        }
    }

    fn offset(self, detents: &[f64]) -> f64 {
        match self {
            Self::Dragging { offset, .. } => offset,
            state => detents[state.snap().min(detents.len() - 1)],
        }
    }
}

/// Maximum sheet height, capped at 92 percent of the viewport.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SheetHeight {
    /// Fit the measured content height.
    Hug,
    /// Use 62 percent of the viewport height, the default.
    #[default]
    Capped,
    /// Use 92 percent of the viewport height.
    Tall,
    /// Height in logical pixels.
    Px(f64),
    /// Height as a fraction of the viewport height.
    Fraction(f64),
}

/// Visible sheet height at an additional draggable detent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SnapPoint {
    /// Visible height in logical pixels.
    Px(f64),
    /// Visible height as a fraction of the viewport height.
    Fraction(f64),
}

/// Dismissal rules for a bottom sheet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogPurpose {
    /// Require an explicit application action; ignore Escape, outside presses and drag dismissal.
    Required,
    /// Allow Escape, while ignoring outside presses and drag dismissal.
    Form,
    /// Allow Escape, outside presses and downward drag dismissal; the default.
    #[default]
    Info,
}

/// Bottom-aligned sheet rebuilt on each opening from the caller's open binding.
///
/// Creates an informational sheet with capped height, a scrim and 16-pixel body padding
/// by default. `padding` is in four-pixel steps, so its default of 4 is 16 px.
#[component]
pub fn bottom_sheet(
    label: Value<String>,
    open: Signal<bool>,
    #[prop(default = DialogPurpose::Info)] purpose: DialogPurpose,
    #[prop(default = SheetHeight::Capped)] height: SheetHeight,
    #[prop(default = Vec::new())] snap_points: Vec<SnapPoint>,
    #[prop(default = true)] scrim: Value<bool>,
    #[prop(default = 4.0)] padding: Value<f64>,
    final_focus: Option<NodeRef>,
    content: Option<ViewFn>,
) -> impl IntoView {
    let modal = scrim.get();
    let padding = (move || padding.get() * 4.0).into_value();
    // The modal overlay automatically restores its opener. An explicit
    // target overrides that restoration after either kind of closure.
    if let Some(target) = final_focus {
        kivact::state::effect_on(open, move |shown| {
            if !shown {
                target.restore_focus();
            }
        });
    }
    let kind = if purpose == DialogPurpose::Required {
        DialogMode::AlertModal
    } else {
        DialogMode::Modal
    };
    let dismiss = DismissOn {
        escape: purpose != DialogPurpose::Required,
        outside: purpose == DialogPurpose::Info,
    };
    let behavior = use_dialog(open, kind).dismiss(dismiss);
    show(open, move || {
        let body_ref = node_ref();
        let body_rect = body_ref.rect();
        let points = snap_points.clone();
        let detents: SheetDetents = Rc::new(move || {
            let viewport = kivact::platform::window_size().get().height;
            let full = sheet_height(height, viewport, body_rect.get().map_or(0.0, |r| r.height));
            (full, offsets(full, viewport, &points))
        });
        let sheet = sheet_machine(detents.clone(), purpose, open);
        let state = sheet.state();
        let drag = sheet_drag(sheet);
        let handle = sheet_handle(drag);
        let body = column(content.as_ref().map(|content| content.call(())))
            .node_ref(body_ref)
            .style(move || style().padding(padding.get()));
        let body = scroll(body)
            .name(label)
            .style(style().flex_grow(1).min_height(0));
        let position_points = snap_points.clone();
        let panel = column((body, handle))
            .style(move || sheet_panel_style(height, body_rect, &position_points, state));
        let scrim = sheet_scrim(modal, detents.clone(), state, dismiss, open);
        let frame = column((scrim, panel)).style(|| {
            let viewport = kivact::platform::window_size().get();
            style()
                .size(viewport.width, viewport.height)
                .justify_content(Justify::End)
                .align_items(Align::Center)
        });
        let layer = overlay(node_ref(), Placement::Center, frame).viewport_margin(0.0);
        if modal {
            layer
                .with_node(move |node| node.part(behavior.content()))
                .name(label)
                .into_view()
        } else {
            layer
                .with_node(move |node| {
                    node.role(if purpose == DialogPurpose::Required {
                        Role::AlertDialog
                    } else {
                        Role::Dialog
                    })
                })
                .name(label)
                .on_dismiss(move |reason| {
                    if dismiss.escape && reason == DismissReason::Escape {
                        open.set(false);
                    }
                })
                .into_view()
        }
    })
    .into_view()
}

/// Opens one of several independent sheets from a shared step binding.
pub fn bottom_sheet_flow<K: PropValue>(
    label: impl IntoText,
    step: Signal<Option<K>>,
    key: K,
) -> BottomSheet {
    let selected = key.clone();
    bottom_sheet(
        label,
        binding(
            move || step.get().as_ref() == Some(&selected),
            move |open| {
                if open {
                    step.set(Some(key.clone()));
                } else if step.get().as_ref() == Some(&key) {
                    step.set(None);
                }
            },
        ),
    )
}

fn sheet_height(height: SheetHeight, viewport: f64, content: f64) -> f64 {
    let height = match height {
        SheetHeight::Hug => content,
        SheetHeight::Capped => viewport * 0.62,
        SheetHeight::Tall => viewport * 0.92,
        SheetHeight::Px(px) => px,
        SheetHeight::Fraction(fraction) => viewport * fraction,
    };
    height.max(0.0).min(viewport.max(0.0) * 0.92)
}

fn offsets(full: f64, viewport: f64, points: &[SnapPoint]) -> Vec<f64> {
    let mut offsets: Vec<_> = points
        .iter()
        .filter_map(|point| {
            let visible = match point {
                SnapPoint::Px(px) => *px,
                SnapPoint::Fraction(fraction) => viewport * fraction,
            };
            (visible > 0.0 && visible < full).then_some(full - visible)
        })
        .collect();
    offsets.push(0.0);
    offsets.sort_by(f64::total_cmp);
    offsets.dedup_by(|later, earlier| *later - *earlier < 48.0);
    offsets
}

type SheetDetents = Rc<dyn Fn() -> (f64, Vec<f64>)>;

/// The sheet drag, driven by one event for each drag phase.
enum SheetEvent {
    Start,
    Update(DragUpdate<()>),
    End(DragUpdate<()>),
    Cancel,
}

/// Detents and dismissal rules read when an event arrives.
struct SheetMachine {
    detents: SheetDetents,
    purpose: DialogPurpose,
}

fn sheet_machine(
    detents: SheetDetents,
    purpose: DialogPurpose,
    open: Signal<bool>,
) -> MachineHandle<SheetMachine> {
    machine(
        Sheet::Resting { snap: 0 },
        move || SheetMachine {
            detents: detents.clone(),
            purpose,
        },
        move |_, close: bool| {
            if close {
                open.set(false);
            }
        },
    )
}

impl Machine for SheetMachine {
    type State = Sheet;
    type Event = SheetEvent;
    /// `true` closes the caller's open binding.
    type Effect = bool;
    fn transition(&self, state: &Sheet, event: SheetEvent) -> Step<Sheet, SheetEvent, bool> {
        let (full, detents) = (self.detents)();
        let current = state.snap().min(detents.len() - 1);
        match event {
            SheetEvent::Start => Step::to(Sheet::Dragging {
                from: current,
                offset: detents[current],
            }),
            SheetEvent::Update(update) => {
                let start = detents[current];
                let raw = start + SHEET_AXIS.component(update.delta);
                let shortest = detents[detents.len() - 1];
                let dismiss_line = shortest + (full - shortest) * DISMISS_FRACTION;
                let offset = if raw < 0.0 {
                    resist_overshoot(raw, OVERSHOOT_RESISTANCE, OVERSHOOT_LIMIT)
                } else if raw < dismiss_line {
                    pull_to_snap(raw, &detents, SNAP_PULL_RADIUS)
                } else {
                    raw
                };
                Step::to(Sheet::Dragging {
                    from: current,
                    offset,
                })
            }
            SheetEvent::End(update) if update.phase == DragPhase::End => {
                let start = detents[current];
                let travel = SHEET_AXIS.component(update.delta);
                let position = start + travel;
                let projected = if SHEET_AXIS
                    .past_slop(update.delta, PROJECTION_MIN_TRAVEL)
                    .is_some()
                {
                    let speed = SHEET_AXIS.component(update.velocity);
                    project_release(position, speed, PROJECTION_DECAY)
                } else {
                    position
                };
                let shortest = detents[detents.len() - 1];
                let line = shortest + (full - shortest) * DISMISS_FRACTION;
                if self.purpose == DialogPurpose::Info && travel > 0.0 && projected > line {
                    Step::stay().with(true)
                } else {
                    let (first, candidates) = if travel > 0.0 {
                        (current, &detents[current..])
                    } else {
                        (0, &detents[..=current])
                    };
                    let to =
                        nearest_snap(projected, candidates).map_or(current, |index| first + index);
                    Step::to(Sheet::Settling { to })
                }
            }
            SheetEvent::End(_) => Step::stay(),
            SheetEvent::Cancel => Step::to(Sheet::Resting { snap: current }),
        }
    }
}

fn sheet_drag(sheet: MachineHandle<SheetMachine>) -> kivact::interaction::DragApi<()> {
    use_drag::<()>(DragCoordinates::Window)
        .track_velocity()
        .on_start(move |_| sheet.send(SheetEvent::Start))
        .on_update(move |update: DragUpdate<()>| sheet.send(SheetEvent::Update(update)))
        .on_end(move |update| {
            if update.phase == DragPhase::Cancel {
                sheet.send(SheetEvent::Cancel);
            } else {
                sheet.send(SheetEvent::End(update));
            }
        })
}

fn sheet_panel_style(
    height: SheetHeight,
    body_rect: kivact::state::Value<Option<kivact::geometry::Rect>>,
    position_points: &[SnapPoint],
    sheet: Value<Sheet>,
) -> kivact::style::Style {
    let viewport = kivact::platform::window_size().get().height;
    let full = sheet_height(height, viewport, body_rect.get().map_or(0.0, |r| r.height));
    let detents = offsets(full, viewport, position_points);
    let offset = sheet.get().offset(&detents);
    let dark = use_theme().get().mode() == ThemeMode::Dark;
    let base = shadowed(body_text(style()), shadows::high(dark))
        .width(pct(100))
        .max_width(640)
        .max_height(pct(92))
        .border_top_left_radius(12)
        .border_top_right_radius(12)
        .background(c::BG_SURFACE)
        .border_top(1, c::BORDER)
        .border_left(1, c::BORDER)
        .border_right(1, c::BORDER)
        .overflow(Overflow::Hidden)
        .translate_y(offset);
    let base = if height == SheetHeight::Hug {
        base
    } else {
        base.height(full)
    };
    if matches!(sheet.get(), Sheet::Dragging { .. }) {
        base
    } else {
        tween(base, 300).enter(style().translate_y(pct(100)))
    }
}

fn sheet_scrim(
    modal: bool,
    scrim_detents: SheetDetents,
    sheet: Value<Sheet>,
    dismiss: DismissOn,
    open: Signal<bool>,
) -> Option<kivact::view::Node> {
    modal.then(|| {
        node()
            .with_node(move |node| node.role(Role::Generic))
            .style(move || {
                let (full, detents) = scrim_detents();
                let offset = sheet.get().offset(&detents);
                let shortest = detents[detents.len() - 1];
                let visible = full - shortest;
                let opacity = if detents.len() > 1 && visible <= full * 0.25 {
                    let previous = detents[detents.len() - 2];
                    1.0 - 0.7 * ((offset - previous) / (shortest - previous)).clamp(0.0, 1.0)
                } else if visible > 0.0 {
                    1.0 - ((offset - shortest) / (visible * 0.4)).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                let base = style()
                    .position(Position::Absolute)
                    .inset(0)
                    .background(c::OVERLAY_SCRIM)
                    .opacity(opacity);
                if matches!(sheet.get(), Sheet::Dragging { .. }) {
                    base
                } else {
                    tween(base, 300).enter(style().opacity(0.0))
                }
            })
            .on_press(move || {
                if dismiss.outside {
                    open.set(false);
                }
            })
    })
}

fn sheet_handle(drag: kivact::interaction::DragApi<()>) -> kivact::view::Node {
    node()
        .with_node(move |node| node.part(drag.source(())))
        .child(node().style(|| style().size(36, 4).border_radius(2).background(c::BORDER)))
        .style(
            style()
                .position(Position::Absolute)
                .top(0)
                .left(0)
                .right(0)
                .height(24)
                .align_items(Align::Center)
                .justify_content(Justify::Center),
        )
}
