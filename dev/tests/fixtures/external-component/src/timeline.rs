//! Role: Range-selectable bar chart timeline component.
//!
//! `Timeline`: a bar chart over buckets (time slots, samples) with a range
//! selection made by dragging with the mouse.
//!
//! Built on `canvas` + `Draw::region`: the selection handles and the
//! selected span are hit regions with semantics (flat a11y leaves). The
//! host does the hit test; `PointerEvent::region()` tells the handler which
//! handle was grabbed. Keyboard: ←/→ move the selection, Shift+←/→ resize
//! it, Home/End jump, Escape clears. Assistive technology: Increment and
//! Decrement move the selection.
//!
//! ```
//! use kivact::prelude::*;
//! use kivact::draw::{Point};
//! use kivact_external_component::{Span, timeline};
//!
//! use kivact::signal;
//!
//! let screen = kivact::test::render(|| {
//!     let sel = signal(None::<Span>);
//!     let series = signal(vec![0.2_f32, 0.8, 0.5, 1.0]);
//!     timeline("Load", series, sel).test_id("load")
//! });
//! let t = screen.get_by_role(Role::Canvas, "Load");
//! t.drag(Point::new(10.0, 20.0), Point::new(250.0, 20.0));
//! assert_eq!(t.value().as_deref(), Some("0 – 1 (2 of 4)"));
//! ```

use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use kivact::view::View;
use std::rc::Rc;

use kivact::draw::{Draw, Rect, RegionId, Size};
use kivact::prelude::*;
use kivact::style::Color;
use kivact::view::{A11yAction, Handled, KeyEvent, PointerEvent};
use kivact::view::{Modifiers, PointerPhase};

use crate::skin::{self, look};

/// A selected bucket range: `start..end` (end exclusive, never empty).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    /// First bucket inside the span.
    pub start: usize,
    /// First bucket after the span.
    pub end: usize,
}

impl Span {
    /// The span from bucket `a` to bucket `b`, both included, in either order.
    pub fn new(a: usize, b: usize) -> Span {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        Span {
            start: lo,
            end: hi + 1,
        }
    }

    /// Number of buckets.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Whether the span has no bucket (never true for a span from `new`).
    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// Whether bucket `i` is inside the span.
    pub fn contains(&self, i: usize) -> bool {
        i >= self.start && i < self.end
    }

    /// Last bucket inside the span.
    pub fn last(&self) -> usize {
        self.end.saturating_sub(1)
    }
}

/// Region id of the start handle inside the canvas (local to each
/// timeline instance).
pub const START_HANDLE: RegionId = RegionId(1);
/// Region id of the end handle.
pub const END_HANDLE: RegionId = RegionId(2);
/// Region id of the selected span.
pub const SPAN_REGION: RegionId = RegionId(3);

const HANDLE_W: f32 = 10.0;

/// Builder of a timeline. Built by [`timeline`].
pub struct Timeline {
    root: Node,
    series: Value<Vec<f32>>,
    selection: Signal<Option<Span>>,
    bucket_label: Rc<dyn Fn(usize) -> String>,
}

/// A timeline over `series` (one value per bucket, any scale) whose range
/// selection is `selection`. `label` is the accessible name. Its node is
/// located at the call of `timeline`.
#[track_caller]
pub fn timeline(
    label: impl IntoText,
    series: impl IntoValue<Vec<f32>>,
    selection: Signal<Option<Span>>,
) -> Timeline {
    let look = look();
    Timeline {
        root: node()
            .role(Role::Canvas)
            .name(label)
            .focusable(true)
            .style(style().height(64).grow(0).shrink(0))
            .style(move || skin::focus_ring(&current_theme()))
            .style(move || {
                let t = &current_theme();
                match look() {
                    skin::Skin::Business => style().border(1, ColorToken::Border),
                    skin::Skin::Console => {
                        style().radius(t.radius().sm).border(1, ColorToken::Border)
                    }
                }
            }),
        series: series.into_value(),
        selection,
        bucket_label: Rc::new(|i| i.to_string()),
    }
}

impl Timeline {
    /// How buckets are named in semantics ("10:05", "sample 3", ...).
    pub fn bucket_label(mut self, f: impl Fn(usize) -> String + 'static) -> Self {
        self.bucket_label = Rc::new(f);
        self
    }

    /// Height in logical px (default 64). The width follows the parent.
    pub fn height(self, h: f32) -> Self {
        self.style(style().height(h))
    }
}

impl NodeBuilder for Timeline {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Timeline {
    fn into_view(self) -> View {
        timeline_view(self).into_view()
    }
}

/// Bars of `values` scaled to their maximum, filling `size`. Shared by the
/// timeline canvas and by local draw layers (e.g. list-row sparklines).
pub fn sparkline(d: &mut Draw, size: Size, values: &[f32], color: ColorToken) {
    let n = values.len();
    if n == 0 || size.width <= 0.0 {
        return;
    }
    let max = values.iter().copied().fold(f32::EPSILON, f32::max);
    let w = size.width / n as f32;
    let gap = if w > 3.0 { 1.0 } else { 0.0 };
    for (i, v) in values.iter().enumerate() {
        let h = (v / max).clamp(0.0, 1.0) * (size.height - 2.0);
        d.fill_rect(
            Rect::new(i as f32 * w, size.height - h, (w - gap).max(0.5), h),
            color,
        );
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Drag {
    Start,
    End,
    Move { grab: usize, from: Span },
    New { anchor: usize },
}

fn timeline_view(p: Timeline) -> impl IntoView {
    let Timeline {
        root,
        series,
        selection,
        bucket_label,
    } = p;
    let buckets = memo(move || series.with(|s| s.len()));
    // Last laid-out size, written by the draw closure (a Var may be written
    // from pure contexts: it is an untracked cache).
    let size = var(Size::default());
    let drag = var(None::<Drag>);
    let bl = bucket_label;
    let names = move |s: Span| (bl(s.start), bl(s.last()));
    let bucket_at = move |x: f32| -> usize {
        let n = buckets.get_untracked().max(1);
        let w = size.get().width.max(1.0);
        ((x / w * n as f32).floor().max(0.0) as usize).min(n - 1)
    };
    let set = move |s: Option<Span>| selection.set(s);
    let shift = move |by: i64, resize: bool| {
        let n = buckets.get_untracked() as i64;
        if n == 0 {
            return;
        }
        let cur = selection.get_untracked();
        let next = match cur {
            None => Span::new(0, 0),
            Some(s) if resize => {
                let end = (s.end as i64 + by).clamp(s.start as i64 + 1, n) as usize;
                Span {
                    start: s.start,
                    end,
                }
            }
            Some(s) => {
                let len = s.len() as i64;
                let start = (s.start as i64 + by).clamp(0, n - len) as usize;
                Span {
                    start,
                    end: start + len as usize,
                }
            }
        };
        set(Some(next));
    };
    let value_names = names.clone();
    let value = move || match selection.get() {
        Some(s) => {
            let (a, b) = value_names(s);
            format!("{a} – {b} ({} of {})", s.len(), buckets.get())
        }
        None => format!("no selection ({} buckets)", buckets.get()),
    };
    let draw = move |d: &mut Draw, sz: Size| {
        size.set(sz);
        let n = buckets.get();
        d.fill_rect(sz.to_rect(), ColorToken::SurfaceAlt);
        series.with(|s| sparkline(d, sz, s, ColorToken::Accent));
        let Some(sel) = selection.get() else { return };
        if n == 0 {
            return;
        }
        let w = sz.width / n as f32;
        let x0 = sel.start as f32 * w;
        let x1 = sel.end as f32 * w;
        let span = Rect::new(x0, 0.0, (x1 - x0).max(1.0), sz.height);
        d.fill_rect(span, Color::token(ColorToken::Selection));
        d.stroke_rect(span, 1.0, ColorToken::Focus);
        let hs = Rect::new(x0 - HANDLE_W / 2.0, 0.0, HANDLE_W, sz.height);
        let he = Rect::new(x1 - HANDLE_W / 2.0, 0.0, HANDLE_W, sz.height);
        for h in [hs, he] {
            d.rounded_rect(
                Rect::new(h.x + 3.0, 4.0, HANDLE_W - 6.0, sz.height - 8.0),
                2.0,
                ColorToken::Focus,
            );
        }
        // Regions: later regions win the hit test, so the handles come last.
        d.region(SPAN_REGION, span);
        d.region(START_HANDLE, hs);
        d.region(END_HANDLE, he);
    };
    let span_names = names.clone();
    let start_names = names.clone();
    let end_names = names;
    let root = root
        .region(
            SPAN_REGION,
            kivact::primitive::part(|state| state)
                .role(Role::Group)
                .name("Selected range")
                .value_text(move || {
                    selection.get().map_or_else(String::new, |span| {
                        let (start, end) = span_names(span);
                        format!("{start} – {end}")
                    })
                }),
        )
        .region(
            START_HANDLE,
            kivact::primitive::part(|state| state)
                .role(Role::Slider)
                .name("Selection start")
                .value_text(move || {
                    selection
                        .get()
                        .map_or_else(String::new, |span| start_names(span).0)
                }),
        )
        .region(
            END_HANDLE,
            kivact::primitive::part(|state| state)
                .role(Role::Slider)
                .name("Selection end")
                .value_text(move || {
                    selection
                        .get()
                        .map_or_else(String::new, |span| end_names(span).1)
                }),
        );
    root.draw(draw)
        .value_text(value)
        .on_pointer(move |e: PointerEvent| {
            let b = bucket_at(e.pos.x);
            match e.phase {
                PointerPhase::Down => {
                    let cur = selection.get_untracked();
                    let mode = match (e.region(), cur) {
                        (Some(r), Some(_)) if r == START_HANDLE => Drag::Start,
                        (Some(r), Some(_)) if r == END_HANDLE => Drag::End,
                        (Some(r), Some(s)) if r == SPAN_REGION => Drag::Move { grab: b, from: s },
                        _ => {
                            set(Some(Span::new(b, b)));
                            Drag::New { anchor: b }
                        }
                    };
                    drag.set(Some(mode));
                    Handled::Yes
                }
                PointerPhase::Move => {
                    let Some(mode) = drag.get() else {
                        return Handled::No;
                    };
                    let cur = selection.get_untracked();
                    let n = buckets.get_untracked();
                    let next = match (mode, cur) {
                        (Drag::Start, Some(s)) => Span::new(b, s.last()),
                        (Drag::End, Some(s)) => Span::new(s.start, b),
                        (Drag::Move { grab, from }, _) => {
                            let len = from.len();
                            let start = (from.start as i64 + b as i64 - grab as i64)
                                .clamp(0, (n - len) as i64)
                                as usize;
                            Span {
                                start,
                                end: start + len,
                            }
                        }
                        (Drag::New { anchor }, _) => Span::new(anchor, b),
                        (_, None) => Span::new(b, b),
                    };
                    set(Some(next));
                    Handled::Yes
                }
                PointerPhase::Up | PointerPhase::Cancel => {
                    let was = drag.get().is_some();
                    drag.set(None);
                    if was { Handled::Yes } else { Handled::No }
                }
                _ => Handled::No,
            }
        })
        .on_key(move |k: KeyEvent| {
            let resize = k.modifiers
                == Modifiers {
                    shift: true,
                    ..Modifiers::default()
                };
            match k.key {
                Key::ArrowRight => shift(1, resize),
                Key::ArrowLeft => shift(-1, resize),
                Key::Home => shift(-(buckets.get_untracked() as i64), false),
                Key::End => shift(buckets.get_untracked() as i64, false),
                Key::Escape if selection.get_untracked().is_some() => set(None),
                _ => return Handled::No,
            }
            Handled::Yes
        })
        .on_a11y(A11yAction::Increment, move || shift(1, false))
        .on_a11y(A11yAction::Decrement, move || shift(-1, false))
}
