//! Role: Present a bottom-aligned sheet with draggable detents.
use crate::recipes_dialog::{body_text, motion, shadows};
use crate::theme_access::tokens;
use kivact::control::NodeBuilder;
use kivact::control::{column, node, scroll};
use kivact::interaction::{Dismiss, DragCoordinates, DragPhase, DragUpdate, use_dialog, use_drag};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Justify, Overflow, Position, pct, style};
use kivact::ui::DialogKind;
use kivact::view::{
    DismissReason, IntoText, IntoView, NodeRef, Placement, Role, View, node_ref, overlay, show,
};
use kivact::{Binding, binding, signal, var};
use std::rc::Rc;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SheetHeight {
    Hug,
    #[default]
    Capped,
    Tall,
    Px(f32),
    Fraction(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SnapPoint {
    Px(f32),
    Fraction(f32),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogPurpose {
    Required,
    Form,
    #[default]
    Info,
}

pub struct BottomSheet {
    label: kivact::Value<String>,
    open: Binding<bool>,
    purpose: DialogPurpose,
    height: SheetHeight,
    snap_points: Vec<SnapPoint>,
    scrim: bool,
    padding: f32,
    final_focus: Option<NodeRef>,
    content: Option<Rc<dyn Fn() -> View>>,
}

pub fn bottom_sheet(label: impl IntoText, open: impl Into<Binding<bool>>) -> BottomSheet {
    BottomSheet {
        label: label.into_text_value(),
        open: open.into(),
        purpose: DialogPurpose::Info,
        height: SheetHeight::Capped,
        snap_points: Vec::new(),
        scrim: true,
        padding: 16.0,
        final_focus: None,
        content: None,
    }
}

/// Opens one of several independent sheets from a shared step binding.
pub fn bottom_sheet_flow<K: Clone + PartialEq + 'static>(
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
    pub fn purpose(mut self, purpose: DialogPurpose) -> Self {
        self.purpose = purpose;
        self
    }

    pub fn height(mut self, height: SheetHeight) -> Self {
        self.height = height;
        self
    }

    pub fn snap_points(mut self, points: impl IntoIterator<Item = SnapPoint>) -> Self {
        self.snap_points = points.into_iter().collect();
        self
    }

    pub fn scrim(mut self, scrim: bool) -> Self {
        self.scrim = scrim;
        self
    }

    pub fn padding(mut self, step: u8) -> Self {
        self.padding = f32::from(step) * 4.0;
        self
    }

    pub fn final_focus(mut self, target: NodeRef) -> Self {
        self.final_focus = Some(target);
        self
    }

    pub fn content<V: IntoView>(mut self, content: impl Fn() -> V + 'static) -> Self {
        self.content = Some(Rc::new(move || content().into_view()));
        self
    }
}

impl IntoView for BottomSheet {
    fn into_view(self) -> View {
        let open = self.open;
        let purpose = self.purpose;
        let modal = self.scrim;
        let target = self.final_focus;
        let open_with_focus = binding(
            move || open.get(),
            move |value| {
                open.set(value);
                if !value && let Some(target) = target {
                    target.focus();
                }
            },
        );
        let kind = if purpose == DialogPurpose::Required {
            DialogKind::AlertModal
        } else {
            DialogKind::Modal
        };
        let dismiss = Dismiss {
            escape: purpose != DialogPurpose::Required,
            outside: purpose == DialogPurpose::Info,
        };
        let behavior = use_dialog(open_with_focus, kind).dismiss(dismiss);
        show(open, move || {
            let dragged = signal(None::<f32>);
            let selected = signal(0usize);
            let samples = var(Vec::<(Instant, f32)>::new());
            let body_ref = node_ref();
            let body_rect = body_ref.rect();
            let height = self.height;
            let points = self.snap_points.clone();
            let detents = Rc::new(move || {
                let viewport = kivact::platform::window_size().height;
                let full =
                    sheet_height(height, viewport, body_rect.get().map_or(0.0, |r| r.height));
                (full, offsets(full, viewport, &points))
            });
            let update_detents = detents.clone();
            let scrim_detents = detents.clone();
            let end_detents = detents.clone();
            let drag = use_drag::<()>(DragCoordinates::Window)
                .on_start(move |_| {
                    samples.set(vec![(Instant::now(), 0.0)]);
                    let (_, detents) = detents();
                    dragged.set(Some(detents[selected.get().min(detents.len() - 1)]));
                })
                .on_update(move |update: DragUpdate<()>| {
                    let now = Instant::now();
                    let mut recent = samples.get();
                    recent.retain(|(time, _)| now.duration_since(*time).as_millis() <= 100);
                    recent.push((now, update.delta.y));
                    samples.set(recent);
                    let (full, detents) = update_detents();
                    let start = detents[selected.get().min(detents.len() - 1)];
                    let raw = start + update.delta.y;
                    let shortest = detents[detents.len() - 1];
                    let dismiss_line = shortest + (full - shortest) * 0.4;
                    let offset = if raw < 0.0 {
                        (raw * 0.35).max(-48.0)
                    } else if raw < dismiss_line {
                        let nearest = detents
                            .iter()
                            .copied()
                            .min_by(|a, b| (a - raw).abs().total_cmp(&(b - raw).abs()))
                            .unwrap_or(0.0);
                        let distance = (nearest - raw).abs();
                        let pull = (1.0 - (distance / 40.0).powi(2)).max(0.0);
                        raw + (nearest - raw) * pull
                    } else {
                        raw
                    };
                    dragged.set(Some(offset));
                })
                .on_end(move |update: DragUpdate<()>| {
                    if update.phase == DragPhase::End {
                        let (full, detents) = end_detents();
                        let current = selected.get().min(detents.len() - 1);
                        let start = detents[current];
                        let travel = update.delta.y;
                        let recent = samples.get();
                        let now = Instant::now();
                        let speed =
                            recent
                                .first()
                                .zip(recent.last())
                                .map_or(0.0, |(first, last)| {
                                    let ms = last.0.duration_since(first.0).as_secs_f32() * 1000.0;
                                    if ms > 0.0 && now.duration_since(last.0).as_millis() <= 100 {
                                        (last.1 - first.1) / ms
                                    } else {
                                        0.0
                                    }
                                });
                        let projected = start
                            + travel
                            + if travel.abs() > 48.0 {
                                speed * 499.0
                            } else {
                                0.0
                            };
                        let shortest = detents[detents.len() - 1];
                        let line = shortest + (full - shortest) * 0.4;
                        if purpose == DialogPurpose::Info && travel > 0.0 && projected > line {
                            open_with_focus.set(false);
                        } else {
                            let nearest = detents
                                .iter()
                                .enumerate()
                                .filter(|(index, _)| {
                                    if travel > 0.0 {
                                        *index >= current
                                    } else {
                                        *index <= current
                                    }
                                })
                                .min_by(|a, b| {
                                    (a.1 - projected).abs().total_cmp(&(b.1 - projected).abs())
                                });
                            if let Some((index, _)) = nearest {
                                selected.set(index);
                            }
                        }
                    }
                    dragged.set(None);
                });
            let handle = node()
                .part(drag.source(()))
                .child(node().style(|| {
                    let p = tokens();
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
                );
            let body = column(self.content.as_ref().map(|content| content()))
                .node_ref(body_ref)
                .style(style().padding(self.padding));
            let body = scroll(body)
                .name(self.label)
                .style(style().grow(1).min_height(0));
            let position_points = self.snap_points.clone();
            let panel = column((body, handle)).style(move || {
                let p = tokens();
                let viewport = kivact::platform::window_size().height;
                let full =
                    sheet_height(height, viewport, body_rect.get().map_or(0.0, |r| r.height));
                let detents = offsets(full, viewport, &position_points);
                let offset = dragged
                    .get()
                    .unwrap_or(detents[selected.get().min(detents.len() - 1)]);
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
                if dragged.get().is_some() {
                    base
                } else {
                    motion(base, &p, 300).enter(style().translate_y(pct(100)))
                }
            });
            let scrim = modal.then(|| {
                node()
                    .style(move || {
                        let p = tokens();
                        let (full, detents) = scrim_detents();
                        let offset = dragged
                            .get()
                            .unwrap_or(detents[selected.get().min(detents.len() - 1)]);
                        let shortest = detents[detents.len() - 1];
                        let visible = full - shortest;
                        let opacity = if detents.len() > 1 && visible <= full * 0.25 {
                            let previous = detents[detents.len() - 2];
                            1.0 - 0.7
                                * ((offset - previous) / (shortest - previous)).clamp(0.0, 1.0)
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
                        if dragged.get().is_some() {
                            base
                        } else {
                            motion(base, &p, 300).enter(style().opacity(0.0))
                        }
                    })
                    .on_press(move || {
                        if dismiss.outside {
                            open_with_focus.set(false);
                        }
                    })
            });
            let frame = column((scrim, panel)).style(
                style()
                    .width(pct(100))
                    .height(pct(100))
                    .justify_content(Justify::End)
                    .align_items(Align::Center),
            );
            let layer = overlay(node_ref(), Placement::Center, frame).viewport_margin(0.0);
            if modal {
                layer.part(behavior.content()).name(self.label).into_view()
            } else {
                layer
                    .role(if purpose == DialogPurpose::Required {
                        Role::AlertDialog
                    } else {
                        Role::Dialog
                    })
                    .name(self.label)
                    .on_dismiss(move |reason| {
                        if dismiss.escape && reason == DismissReason::Escape {
                            open_with_focus.set(false);
                        }
                    })
                    .into_view()
            }
        })
        .into_view()
    }
}

fn sheet_height(height: SheetHeight, viewport: f32, content: f32) -> f32 {
    let height = match height {
        SheetHeight::Hug => content,
        SheetHeight::Capped => viewport * 0.62,
        SheetHeight::Tall => viewport * 0.92,
        SheetHeight::Px(px) => px,
        SheetHeight::Fraction(fraction) => viewport * fraction,
    };
    height.max(0.0).min(viewport.max(0.0) * 0.92)
}

fn offsets(full: f32, viewport: f32, points: &[SnapPoint]) -> Vec<f32> {
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
    offsets.sort_by(f32::total_cmp);
    offsets.dedup_by(|later, earlier| *later - *earlier < 48.0);
    offsets
}
