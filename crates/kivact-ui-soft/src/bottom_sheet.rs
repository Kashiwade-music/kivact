//! Role: Present a bottom-aligned sheet with draggable detents.
use crate::recipes_dialog::{body_text, motion, shadows};

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
use kivact::state::Binding;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::PropValue;
use kivact::state::binding;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::Overflow;
use kivact::style::Position;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::DismissReason;
use kivact::view::Element;
use kivact::view::IntoView;
use kivact::view::IntoViewFn;
use kivact::view::NodeRef;
use kivact::view::Placement;
use kivact::view::Role;
use kivact::view::View;
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
pub struct BottomSheet {
    label: kivact::state::Value<String>,
    open: Binding<bool>,
    purpose: DialogPurpose,
    height: SheetHeight,
    snap_points: Vec<SnapPoint>,
    scrim: kivact::state::Value<bool>,
    padding: kivact::state::Value<f64>,
    // xtask-allow(state-fields): Optional caller-selected focus destination, not a pending focus phase.
    final_focus: Option<NodeRef>,
    content: Option<ViewFn>,
}

/// Create an informational sheet with capped height, a scrim and 16-pixel body padding.
pub fn bottom_sheet(label: impl IntoText, open: impl IntoBinding<bool>) -> BottomSheet {
    BottomSheet {
        label: label.into_text_value(),
        open: open.into_binding(),
        purpose: DialogPurpose::Info,
        height: SheetHeight::Capped,
        snap_points: Vec::new(),
        scrim: true.into_value(),
        padding: 16.0.into_value(),
        final_focus: None,
        content: None,
    }
}

/// Opens one of several independent sheets from a shared step binding.
pub fn bottom_sheet_flow<K: PropValue>(
    label: impl IntoText,
    step: Binding<Option<K>>,
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

impl BottomSheet {
    /// Choose fixed dismissal rules; informational by default.
    pub fn purpose(mut self, purpose: DialogPurpose) -> Self {
        self.purpose = purpose;
        self
    }

    /// Choose the maximum sheet height; capped by default.
    pub fn height(mut self, height: SheetHeight) -> Self {
        self.height = height;
        self
    }

    /// Set extra visible-height detents; the fully open position is always included.
    pub fn snap_points(mut self, points: impl IntoIterator<Item = SnapPoint>) -> Self {
        self.snap_points = points.into_iter().collect();
        self
    }

    /// Enable the scrim; its value at mounting also chooses modal presentation.
    pub fn scrim(mut self, scrim: impl IntoValue<bool>) -> Self {
        self.scrim = scrim.into_value();
        self
    }

    /// Set body padding in four-pixel spacing steps; defaults to four steps.
    pub fn padding(mut self, step: impl IntoValue<f64>) -> Self {
        let step = step.into_value();
        self.padding = (move || step.get() * 4.0).into_value();
        self
    }

    /// Restore focus to this target after closing instead of relying on the default opener.
    pub fn final_focus(mut self, target: NodeRef) -> Self {
        self.final_focus = Some(target);
        self
    }

    /// Build body content anew for each opening; no content is installed by default.
    pub fn content<V: IntoView>(mut self, content: impl IntoViewFn<V>) -> Self {
        self.content = Some(content.into_view_fn());
        self
    }
}

impl IntoView for BottomSheet {
    fn into_view(self) -> View {
        let open = self.open;
        let purpose = self.purpose;
        let modal = self.scrim.get();
        // The modal overlay automatically restores its opener. An explicit
        // target overrides that restoration after either kind of closure.
        if let Some(target) = self.final_focus {
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
            let sheet = signal(Sheet::Resting { snap: 0 });
            let body_ref = node_ref();
            let body_rect = body_ref.rect();
            let height = self.height;
            let points = self.snap_points.clone();
            let detents: SheetDetents = Rc::new(move || {
                let viewport = kivact::platform::window_size().get().height;
                let full =
                    sheet_height(height, viewport, body_rect.get().map_or(0.0, |r| r.height));
                (full, offsets(full, viewport, &points))
            });
            let drag = sheet_drag(sheet, detents.clone(), purpose, open);
            let handle = sheet_handle(drag);
            let body = column(self.content.as_ref().map(|content| content.call(())))
                .node_ref(body_ref)
                .style(move || style().padding(self.padding.get()));
            let body = scroll(body)
                .name(self.label)
                .style(style().grow(1).min_height(0));
            let position_points = self.snap_points.clone();
            let panel = column((body, handle))
                .style(move || sheet_panel_style(height, body_rect, &position_points, sheet));
            let scrim = sheet_scrim(modal, detents.clone(), sheet, dismiss, open);
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
                    .name(self.label)
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
                    .name(self.label)
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
fn sheet_drag(
    sheet: kivact::state::Signal<Sheet>,
    detents: SheetDetents,
    purpose: DialogPurpose,
    open: Binding<bool>,
) -> kivact::interaction::DragApi<()> {
    let update_detents = detents.clone();
    let end_detents = detents.clone();
    use_drag::<()>(DragCoordinates::Window)
        .track_velocity()
        .on_start(move |_| {
            let (_, detents) = detents();
            let from = sheet.get().snap().min(detents.len() - 1);
            sheet.set(Sheet::Dragging {
                from,
                offset: detents[from],
            });
        })
        .on_update(move |update: DragUpdate<()>| {
            let (full, detents) = update_detents();
            let start = detents[sheet.get().snap().min(detents.len() - 1)];
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
            sheet.set(Sheet::Dragging {
                from: sheet.get().snap(),
                offset,
            });
        })
        .on_end(move |update| sheet_release(update, sheet, &end_detents, purpose, open))
}

fn sheet_release(
    update: DragUpdate<()>,
    sheet: kivact::state::Signal<Sheet>,
    end_detents: &SheetDetents,
    purpose: DialogPurpose,
    open: Binding<bool>,
) {
    if update.phase == DragPhase::End {
        let (full, detents) = end_detents();
        let current = sheet.get().snap().min(detents.len() - 1);
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
        if purpose == DialogPurpose::Info && travel > 0.0 && projected > line {
            open.set(false);
        } else {
            let (first, candidates) = if travel > 0.0 {
                (current, &detents[current..])
            } else {
                (0, &detents[..=current])
            };
            if let Some(index) = nearest_snap(projected, candidates) {
                sheet.set(Sheet::Settling { to: first + index });
            }
        }
    }
    if update.phase == DragPhase::Cancel {
        sheet.set(Sheet::Resting {
            snap: sheet.get().snap(),
        });
    }
}

fn sheet_panel_style(
    height: SheetHeight,
    body_rect: kivact::state::Value<Option<kivact::geometry::Rect>>,
    position_points: &[SnapPoint],
    sheet: kivact::state::Signal<Sheet>,
) -> kivact::style::Style {
    let p = kivact::style::use_tokens::<crate::Tokens>();
    let viewport = kivact::platform::window_size().get().height;
    let full = sheet_height(height, viewport, body_rect.get().map_or(0.0, |r| r.height));
    let detents = offsets(full, viewport, position_points);
    let offset = sheet.get().offset(&detents);
    let base = shadows(body_text(&p), &p.shadows.high)
        .width(pct(100))
        .max_width(640)
        .max_height(pct(92))
        .radius_top_left(12)
        .radius_top_right(12)
        .background(p.colors.bg_surface)
        .border_top(1, p.colors.border)
        .border_left(1, p.colors.border)
        .border_right(1, p.colors.border)
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
        motion(base, &p, 300).enter(style().translate_y(pct(100)))
    }
}

fn sheet_scrim(
    modal: bool,
    scrim_detents: SheetDetents,
    sheet: kivact::state::Signal<Sheet>,
    dismiss: DismissOn,
    open: Binding<bool>,
) -> Option<kivact::view::Node> {
    modal.then(|| {
        node()
            .with_node(move |node| node.role(Role::Generic))
            .style(move || {
                let p = kivact::style::use_tokens::<crate::Tokens>();
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
                    .background(p.colors.overlay_scrim)
                    .opacity(opacity);
                if matches!(sheet.get(), Sheet::Dragging { .. }) {
                    base
                } else {
                    motion(base, &p, 300).enter(style().opacity(0.0))
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
        .child(node().style(|| {
            let p = kivact::style::use_tokens::<crate::Tokens>();
            style().size(36, 4).radius(2).background(p.colors.border)
        }))
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
