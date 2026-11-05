//! Role: Drawer: a modal panel attached to an edge that a swipe dismisses.
//!
//! A sheet (sheet.tsx) is the standard `kivact::ui::dialog` of kind
//! `DialogMode::Drawer(side)`, drawn by the kit's dialog recipes. The drawer
//! stays a component of its own because the swipe moves its panel with the
//! pointer and decides from the distance and the speed whether to close,
//! which a dialog recipe cannot express.

use crate::layers::{description_style, scrim, title_style};

use kivact::interaction::{DragCoordinates, DragPhase, use_dialog, use_drag};
use kivact::motion::{Easing, Transition};
use kivact::prelude::NodeBuilder;
use kivact::view::{column, node, row, scroll, text};

use kivact::custom::clock_now;
use kivact::geometry::Edge;
use kivact::interaction::DialogMode;
use kivact::state::IntoText;
use kivact::state::{Binding, Value};
use kivact::state::{signal, var};
use kivact::style::{Align, Justify, Position, Style, TransitionProp, pct, style};
use kivact::view::{IntoView, Placement, View, node_ref, overlay, show};

/// A modal panel with a swipe handle (drawer.tsx).
///
/// Created by [`drawer`]. Drag the handle toward the edge to dismiss it:
/// past 30 % of the panel's size, or with a quick flick, the drawer closes;
/// otherwise it snaps back. Escape and a press on the scrim close it too.
pub struct Drawer {
    open: Binding<bool>,
    title: Value<String>,
    side: Edge,
    description: Option<Value<String>>,
    content: Option<kivact::view::ViewFn>,
    footer: Option<kivact::view::ViewFn>,
}

/// A drawer titled `title`, shown while `open` is true.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn move_goal(open: Signal<bool>) -> impl IntoView {
///     basic::drawer(open, "Move Goal")
///         .description("Set your daily activity goal.")
///         .content(|| text("350 calories/day"))
///         .footer(|| button("Submit"))
/// }
/// ```
#[track_caller]
pub fn drawer(open: impl kivact::state::IntoBinding<bool>, title: impl IntoText) -> Drawer {
    Drawer {
        open: open.into_binding(),
        title: title.into_text_value(),
        side: Edge::Bottom,
        description: None,
        content: None,
        footer: None,
    }
}

impl Drawer {
    /// The edge of the window the drawer is attached to (default
    /// `Edge::Bottom`); it is swiped away toward that edge. An edge of the
    /// window, as the `Edge` of `DialogMode::Drawer`, not a placement beside
    /// a trigger.
    pub fn side(mut self, side: Edge) -> Self {
        self.side = side;
        self
    }

    /// The explanation under the title; it describes the dialog to assistive technology.
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(text.into_text_value());
        self
    }

    /// The body between the header and the footer.
    ///
    /// `content` is called each time the drawer opens.
    pub fn content<V: IntoView>(mut self, content: impl kivact::view::IntoViewFn<V>) -> Self {
        self.content = Some(content.into_view_fn());
        self
    }

    /// The buttons at the bottom of the drawer, stacked in a column.
    ///
    /// `footer` is called each time the drawer opens.
    pub fn footer<V: IntoView>(mut self, footer: impl kivact::view::IntoViewFn<V>) -> Self {
        self.footer = Some(footer.into_view_fn());
        self
    }
}

/// The share of the panel's size a swipe has to pass to close it.
const CLOSE_FRACTION: f64 = 0.3;
/// The speed in px per millisecond that closes the drawer whatever the distance.
const FLICK_SPEED: f64 = 0.6;

impl Drawer {
    fn horizontal(&self) -> bool {
        matches!(self.side, Edge::Left | Edge::Right)
    }
}

impl IntoView for Drawer {
    fn into_view(self) -> View {
        let horizontal = self.horizontal();
        let Drawer {
            open,
            title,
            side: direction,
            description,
            content,
            footer,
        } = self;
        let behavior = use_dialog(open, DialogMode::Drawer(direction));
        show(behavior.is_open(), move || {
            let surface = behavior.content();
            // How far the panel is pulled toward its edge, in px.
            let pulled = signal(0.0f64);
            let panel_ref = node_ref();
            let rect = panel_ref.rect();
            let last = var((0.0f64, 0u64));
            let speed = var(0.0f64);
            let session = use_drag::<()>(DragCoordinates::Window)
                .on_start(move |_| {
                    last.set((0.0, clock_now()));
                    speed.set(0.0);
                })
                .on_update(move |update: kivact::interaction::DragUpdate<()>| {
                    let toward = match direction {
                        Edge::Bottom => update.delta.y,
                        Edge::Top => -update.delta.y,
                        Edge::Right => update.delta.x,
                        Edge::Left => -update.delta.x,
                    }
                    .max(0.0);
                    let now = clock_now();
                    let (before, at) = last.get();
                    if now > at {
                        let millis = (now - at) as f64 / 1_000_000.0;
                        speed.set((toward - before) / millis);
                    }
                    last.set((toward, now));
                    if update.phase == DragPhase::Cancel {
                        pulled.set(0.0);
                    } else {
                        pulled.set(toward);
                    }
                })
                .on_end(move |update: kivact::interaction::DragUpdate<()>| {
                    let toward = pulled.get();
                    // Not measured yet: no distance is far enough.
                    let far = rect
                        .get()
                        .map(|rect| if horizontal { rect.width } else { rect.height })
                        .is_some_and(|size| size > 0.0 && toward > size * CLOSE_FRACTION);
                    let flick = toward > 4.0 && speed.get() > FLICK_SPEED;
                    if update.phase == DragPhase::End && (far || flick) {
                        open.set(false);
                    } else {
                        pulled.set(0.0);
                    }
                });
            let source = session.source(());
            let swiping = source.state();
            let handle = node()
                .with_node(|root| root.part(source))
                .style(move || handle_style(direction, swiping.get().active));
            let heading = text(title)
                .with_node(|root| root.part(behavior.title()))
                .style(title_style);
            let explanation = description.map(|description| {
                text(description)
                    .with_node(|root| root.part(behavior.description()))
                    .style(description_style)
            });
            let header = column((heading, explanation))
                .style(style().gap(2).padding(16).padding_bottom(0).shrink(0));
            let body = content.as_ref().map(|content| {
                scroll(column(content.call(())).style(style().padding_x(16).padding_top(16)))
                    .style(style().grow(1).min_height(0))
            });
            let bottom = footer.as_ref().map(|footer| {
                column(footer.call(())).style(
                    style()
                        .margin_top(kivact::style::auto())
                        .shrink(0)
                        .gap(8)
                        .padding(16)
                        .padding_top(0),
                )
            });
            let inner =
                column((header, body, bottom)).style(style().grow(1).min_height(0).min_width(0));
            let tail = matches!(direction, Edge::Top | Edge::Left);
            let parts = if tail {
                (None, Some(inner), Some(handle))
            } else {
                (Some(handle), Some(inner), None)
            };
            let swipe_state = swiping;
            let panel_style = move || {
                let moved = pulled.get();
                let s = drawer_panel(direction);
                let (x, y) = match direction {
                    Edge::Bottom => (0.0, moved),
                    Edge::Top => (0.0, -moved),
                    Edge::Right => (moved, 0.0),
                    Edge::Left => (-moved, 0.0),
                };
                let s = s.translate(x, y);
                if swipe_state.get().active {
                    s
                } else {
                    s.transition(
                        TransitionProp::Paint,
                        Transition::tween(200, Easing::EaseOut),
                    )
                }
            };
            let panel = if horizontal {
                row((parts.0, parts.1, parts.2))
            } else {
                column((parts.0, parts.1, parts.2))
            }
            .node_ref(panel_ref)
            .style(panel_style);
            (
                scrim(),
                overlay(node_ref(), Placement::Center, panel)
                    .with_node(|root| root.part(surface))
                    .name(title),
            )
        })
        .into_view()
    }
}

/// `bg-popover text-xs/relaxed` attached to the edge, bordered on its inner side.
fn drawer_panel(direction: Edge) -> Style {
    let p = kivact::style::use_tokens::<crate::Tokens>();
    let base = style()
        .background(p.popover)
        .color(p.popover_foreground)
        .font_size(12)
        .line_height(1.625)
        .position(Position::Fixed);
    match direction {
        Edge::Bottom => base
            .left(0)
            .right(0)
            .bottom(0)
            .max_height(pct(100) - 96)
            .border_top(1, p.border),
        Edge::Top => base
            .left(0)
            .right(0)
            .top(0)
            .max_height(pct(100) - 96)
            .border_bottom(1, p.border),
        Edge::Left => base
            .left(0)
            .top(0)
            .bottom(0)
            .width(pct(75))
            .max_width(384)
            .border_right(1, p.border),
        Edge::Right => base
            .right(0)
            .top(0)
            .bottom(0)
            .width(pct(75))
            .max_width(384)
            .border_left(1, p.border),
    }
}

/// The 12 px strip with the 48 x 4 `bg-muted` bar, grabbed to swipe.
fn handle_style(direction: Edge, active: bool) -> Style {
    let p = kivact::style::use_tokens::<crate::Tokens>();
    let horizontal = matches!(direction, Edge::Left | Edge::Right);
    let (bar_w, bar_h) = if horizontal { (4.0, 48.0) } else { (48.0, 4.0) };
    let s = style()
        .shrink(0)
        .direction(kivact::style::Direction::Row)
        .cursor(if active {
            kivact::style::Cursor::Grabbing
        } else {
            kivact::style::Cursor::Grab
        })
        .paint(move |d, size| {
            let (x, y) = match direction {
                Edge::Bottom => ((size.width - bar_w) / 2.0, size.height - bar_h),
                Edge::Top => ((size.width - bar_w) / 2.0, 0.0),
                Edge::Left => (0.0, (size.height - bar_h) / 2.0),
                Edge::Right => (size.width - bar_w, (size.height - bar_h) / 2.0),
            };
            d.fill_rect(kivact::geometry::Rect::new(x, y, bar_w, bar_h), p.muted);
        });
    if horizontal {
        s.width(12).align_self(Align::Stretch)
    } else {
        s.height(12)
            .width(pct(100))
            .justify_content(Justify::Center)
    }
}
