//! Role: Build the carousel track, paging controls and keyboard behavior.

use crate::carousel::look::CarouselSurfaceState;
use crate::carousel::look::CarouselTheme;
use kivact_core::custom::Caller;
use kivact_core::geometry::Point;
use kivact_core::state::{Signal, Value, effect, memo};
use kivact_core::style::{FlexDirection, Overflow, pct, style, use_theme};
use kivact_core::view::{
    Children, Element, Handled, IntoView, Key, Node, NodeRef, Orientation, ReadingDirection, Role,
    ScrollBehavior, View, node, node_ref, use_direction,
};
use kivact_interaction::{CarouselApi, CarouselState, Snap, use_carousel};
use kivact_macros::component;
use std::rc::Rc;

/// A slide viewport and previous/next controls.
///
/// Show slides in caller-owned index order. Arrow keys follow orientation and reading direction.
/// Its name is the message `carousel.label` ("Carousel"), its buttons `carousel.previous` and `carousel.next`, and each slide `carousel.slide` ("Slide {index} / {count}").
#[component]
pub fn carousel(
    index: Signal<usize>,
    slides: Children,
    #[prop(default = Orientation::Horizontal)] orientation: Value<Orientation>,
    #[prop(default = Snap::Mandatory)] snap: Value<Snap>,
    #[prop(default = CarouselMode::Translate)] mode: CarouselMode,
    #[prop(default = false)] looping: bool,
    items_per_view: Option<Value<f64>>,
    gap: Option<Value<f64>>,
    padding: Option<Value<f64>>,
    #[prop(default = true)] buttons: Value<bool>,
    #[prop(default = true)] edge_fade: Value<bool>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let slides = slides.into_views();
    let references = Rc::new((0..slides.len()).map(|_| node_ref()).collect::<Vec<_>>());
    let edges = (references.first().copied(), references.last().copied());
    let setup = Setup {
        index,
        count: slides.len(),
        orientation,
        snap,
        mode,
        looping,
        items_per_view,
        gap,
        padding,
        buttons,
        edge_fade,
    };
    let c = prepare(setup, references.clone(), caller.disabled());
    if c.mode == CarouselMode::Scroll {
        follow_scroll(c, references.clone());
        open_at_index(c, references.clone());
    }
    let items = slide_items(c, &variants, slides, &references);
    let content = slide_track(c, &variants, items);
    let viewport = slide_viewport(c, &variants, content, edges);
    let previous = navigation_button(c, &variants, references.clone(), edges, false);
    let next = navigation_button(c, &variants, references.clone(), edges, true);
    crate::layer::with_base_recipe_style(
        node().name(kivact_core::i18n::message("carousel.label", "Carousel")),
        |r: &CarouselTheme| r.root_recipe(),
        &variants,
        move || c.recipe_state(),
    )
    .with_node(|n| n.role(Role::Group))
    .focusable(move || !c.disabled.get())
    .on_key(move |event| keyboard(c, &references, event))
    .child(viewport)
    .child((previous, next))
}

/// The props the body reads once, before the layout is built.
struct Setup {
    index: Signal<usize>,
    count: usize,
    orientation: Value<Orientation>,
    snap: Value<Snap>,
    mode: CarouselMode,
    looping: bool,
    items_per_view: Option<Value<f64>>,
    gap: Option<Value<f64>>,
    padding: Option<Value<f64>>,
    buttons: Value<bool>,
    edge_fade: Value<bool>,
}

/// What the scroll follow and the machine's scroll read: the viewport, the
/// axis, the reading direction, the snap and the padding.
#[derive(Clone, Copy)]
struct SlideView {
    viewport: NodeRef,
    vertical: Value<bool>,
    rtl: ReadingDirection,
    snap: Value<Snap>,
    padding: Value<f64>,
}

#[derive(Clone, Copy)]
struct Layout {
    api: CarouselApi,
    navigation: Value<CarouselState>,
    mode: CarouselMode,
    looping: bool,
    view: SlideView,
    orientation: Value<Orientation>,
    track: NodeRef,
    count: usize,
    last: Value<usize>,
    shown: Value<usize>,
    disabled: Value<bool>,
    buttons: Value<bool>,
    edge_fade: Value<bool>,
    per_view: Option<Value<f64>>,
    gap: Option<Value<f64>>,
}

impl Layout {
    fn padding(self) -> f64 {
        self.view.padding.get()
    }
    fn gap(self) -> f64 {
        self.gap.map_or_else(
            || crate::carousel::look::carousel_gap(&kivact_core::style::use_theme().get()),
            |v| v.get(),
        )
    }
    fn visible(self) -> f64 {
        self.per_view.map_or(1.0, |v| v.get())
    }
    fn recipe_state(self) -> CarouselSurfaceState {
        CarouselSurfaceState {
            state: self.navigation.get(),
            orientation: self.orientation.get(),
            direction: self.view.rtl,
        }
    }
}

/// Prepare immutable layout inputs and authoritative navigation handles.
fn prepare(setup: Setup, references: Rc<Vec<NodeRef>>, disabled: Option<Value<bool>>) -> Layout {
    let disabled = memo(move || disabled.is_some_and(|v| v.get()));
    let count = setup.count;
    let orientation = setup.orientation;
    let vertical = memo(move || orientation.get() == Orientation::Vertical);
    let padding = setup.padding;
    let view = SlideView {
        viewport: node_ref(),
        vertical,
        rtl: use_direction(),
        snap: setup.snap,
        padding: memo(move || {
            padding.map_or_else(
                || crate::carousel::look::carousel_padding(&use_theme().get()),
                |v| v.get(),
            )
        }),
    };
    let api = use_carousel(setup.index, count, move |at| {
        scroll_to_slide(view, &references, at, false)
    });
    let per_view = setup.items_per_view.map(|count| {
        memo(move || {
            let n = count.get();
            if n.is_finite() { n.max(1.0) } else { 1.0 }
        })
    });
    let last =
        memo(move || count.saturating_sub(per_view.map_or(1.0, |v| v.get()).ceil() as usize));
    let shown = memo(move || api.index().get().min(last.get()));
    Layout {
        api,
        navigation: api.state(),
        mode: setup.mode,
        looping: setup.looping,
        view,
        orientation,
        track: node_ref(),
        count,
        last,
        shown,
        disabled,
        buttons: setup.buttons,
        edge_fade: setup.edge_fade,
        per_view,
        gap: setup.gap,
    }
}

/// Navigate one page using measured scrolling or the translated slide index.
fn navigate(c: Layout, references: &[NodeRef], delta: isize) {
    if c.disabled.get_untracked() {
        return;
    }
    if c.mode == CarouselMode::Scroll {
        if let Some(target) = crate::carousel::geometry::page(
            c.view.viewport,
            references,
            crate::carousel::geometry::PageRequest {
                vertical: c.view.vertical.get_untracked(),
                rtl: c.view.rtl == ReadingDirection::Rtl,
                next: delta > 0,
                looping: c.looping,
                snap: c.view.snap.get_untracked(),
                padding: c.padding(),
            },
        ) {
            c.api.go_to(target);
        }
        return;
    }
    let at = c.shown.get_untracked() as isize;
    let period = c.last.get_untracked() as isize + 1;
    c.api.go_to(if c.looping {
        (at + delta).rem_euclid(period)
    } else {
        (at + delta).clamp(0, period - 1)
    } as usize);
}

/// Scroll the viewport so slide `at` starts at its padding edge. The carousel
/// machine calls it for each command and each outside write of the index.
/// `instant` skips the motion; the first placement of a carousel uses it.
fn scroll_to_slide(view: SlideView, references: &[NodeRef], at: usize, instant: bool) {
    let (Some(viewport), Some(slide)) = (
        view.viewport.rect().get_untracked(),
        references.get(at).and_then(|r| r.rect().get_untracked()),
    ) else {
        return;
    };
    let old = view.viewport.scroll_offset().get_untracked();
    let rtl = view.rtl == ReadingDirection::Rtl;
    let padding = view.padding.get_untracked();
    let point = if view.vertical.get_untracked() {
        Point {
            x: old.x,
            y: old.y + slide.y - viewport.y - padding,
        }
    } else {
        Point {
            x: old.x
                + if rtl {
                    slide.right() - viewport.right() + padding
                } else {
                    slide.x - viewport.x - padding
                },
            y: old.y,
        }
    };
    if view.snap.get_untracked() == Snap::None {
        let nearest = nearest_slide(
            &slide_rects(references),
            viewport,
            view.vertical.get_untracked(),
            rtl,
        );
        if nearest == Some(at) {
            return;
        }
    }
    view.viewport.scroll_to(
        point,
        if instant || kivact_core::platform::is_motion_reduced().get_untracked() {
            ScrollBehavior::Instant
        } else {
            ScrollBehavior::Smooth(kivact_core::motion::Easing::EaseOut)
        },
    );
}

/// Place the current slide once, the first time the viewport and that slide
/// are measured, so a carousel opens at its index. The index may be set before
/// the first layout, and the rects arrive after it; the placement is instant.
/// Later measurements do not scroll: only index changes do.
fn open_at_index(c: Layout, references: Rc<Vec<NodeRef>>) {
    let opened = std::cell::Cell::new(false);
    effect(move || {
        let viewport = c.view.viewport.rect().get().is_some();
        let at = c.shown.get_untracked();
        let slide = references.get(at).is_some_and(|r| r.rect().get().is_some());
        if opened.get() || !(viewport && slide) {
            return;
        }
        opened.set(true);
        scroll_to_slide(c.view, &references, at, true);
    });
}

/// Report the slide nearest the viewport's start to the carousel each time
/// the native scroll offset changes. The viewport's and the slides' rects are
/// taken here, at setup: taking a rect asks the host for the node's layout,
/// and the first scroll must find it reported.
fn follow_scroll(c: Layout, navigation_refs: Rc<Vec<NodeRef>>) {
    let viewport = c.view.viewport.rect();
    let slides = slide_rects(&navigation_refs);
    kivact_core::state::effect_on(c.view.viewport.scroll_offset(), move |_: &Point| {
        let Some(view) = viewport.get_untracked() else {
            return;
        };
        let nearest = nearest_slide(
            &slides,
            view,
            c.view.vertical.get_untracked(),
            c.view.rtl == ReadingDirection::Rtl,
        );
        if let Some(at) = nearest {
            c.api.scrolling(at);
        }
    });
}

/// The layout rects of the slides behind `references`.
fn slide_rects(references: &[NodeRef]) -> Vec<Value<Option<kivact_core::geometry::Rect>>> {
    references.iter().map(NodeRef::rect).collect()
}

/// Locate the closest measured slide along the actual reading axis.
fn nearest_slide(
    slides: &[Value<Option<kivact_core::geometry::Rect>>],
    view: kivact_core::geometry::Rect,
    vertical: bool,
    rtl: bool,
) -> Option<usize> {
    slides
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            r.get_untracked().map(|r| {
                (
                    i,
                    if vertical {
                        (r.y - view.y).abs()
                    } else if rtl {
                        (r.right() - view.right()).abs()
                    } else {
                        (r.x - view.x).abs()
                    },
                )
            })
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|p| p.0)
}

/// Size and label each slide using the actual carousel recipe state.
fn slide_items(
    c: Layout,
    variants: &kivact_core::view::Variants,
    slides: Vec<View>,
    refs: &[NodeRef],
) -> Vec<View> {
    slides
        .into_iter()
        .enumerate()
        .map(|(at, item)| {
            crate::layer::with_recipe_style(
                node(),
                |r: &CarouselTheme| r.slide_recipe(),
                variants,
                move || c.recipe_state(),
            )
            .node_ref(refs[at])
            .with_node(|n| n.role(Role::Group))
            .name({
                let count = c.count;
                kivact_core::i18n::message_with(
                    "carousel.slide",
                    "Slide {index} / {count}",
                    move || {
                        [
                            ("index", (at + 1).to_string()),
                            ("count", count.to_string()),
                        ]
                    },
                )
            })
            .style(move || {
                style()
                    .flex_grow(0)
                    .flex_shrink(0)
                    .flex_basis(
                        if c.mode == CarouselMode::Translate || c.per_view.is_some() {
                            pct(100.0 / c.visible()) - c.gap() * (c.visible() - 1.0) / c.visible()
                        } else {
                            kivact_core::style::Length::Auto
                        },
                    )
                    .min_width(0)
                    .min_height(0)
            })
            .child(item)
            .into_view()
        })
        .collect()
}

/// Lay out and translate the shared slide track.
fn slide_track(c: Layout, variants: &kivact_core::view::Variants, items: Vec<View>) -> Node {
    crate::layer::with_recipe_style(
        node(),
        |r: &CarouselTheme| r.track_recipe(),
        variants,
        move || c.recipe_state(),
    )
    .node_ref(c.track)
    .style(move || {
        let mut s = style().flex_direction(if c.view.vertical.get() {
            FlexDirection::Column
        } else if c.view.rtl == ReadingDirection::Rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        });
        if let Some(gap) = c.gap {
            s = s.gap(gap.get());
        }
        if c.mode == CarouselMode::Translate {
            let along = -(c.shown.get() as f64) * 100.0 / c.visible();
            let offset = pct(along) - (c.shown.get() as f64) * c.gap() / c.visible();
            s = if c.view.vertical.get() {
                s.translate(0, offset)
            } else {
                s.translate(
                    if c.view.rtl == ReadingDirection::Rtl {
                        pct(-along) + (c.shown.get() as f64) * c.gap() / c.visible()
                    } else {
                        offset
                    },
                    0,
                )
            };
        }
        s
    })
    .child(items)
}

/// Wrap slides in the translated or native scrolling viewport.
fn slide_viewport(
    c: Layout,
    variants: &kivact_core::view::Variants,
    content: Node,
    edges: (Option<NodeRef>, Option<NodeRef>),
) -> Node {
    crate::layer::with_recipe_style(
        node(),
        |r: &CarouselTheme| r.viewport_recipe(),
        variants,
        move || c.recipe_state(),
    )
    .node_ref(c.view.viewport)
    .style(move || {
        style()
            .min_width(0)
            .min_height(0)
            .padding(c.padding())
            .overflow(if c.mode == CarouselMode::Scroll {
                Overflow::Scroll
            } else {
                Overflow::Hidden
            })
    })
    .style(move || edge_mask(c, edges))
    .on_wheel(move |event| {
        if !c.disabled.get_untracked()
            && c.mode == CarouselMode::Scroll
            && !c.view.vertical.get_untracked()
            && event.modifiers().shift
            && event.delta().x == 0.0
            && event.delta().y != 0.0
            && (crate::carousel::geometry::can_scroll(
                c.view.viewport,
                edges,
                false,
                false,
                true,
                true,
            ))
        {
            c.view.viewport.scroll_by(
                Point {
                    x: event.delta().y,
                    y: 0.0,
                },
                ScrollBehavior::Instant,
            );
            Handled::Yes
        } else {
            Handled::No
        }
    })
    .child(content)
}

/// Project overflow edge fades from current measured slide geometry.
fn edge_mask(c: Layout, edges: (Option<NodeRef>, Option<NodeRef>)) -> kivact_core::style::Style {
    if !c.edge_fade.get() || c.mode != CarouselMode::Scroll {
        return style();
    }
    let t = kivact_core::style::use_theme().get();
    let fade = crate::carousel::look::carousel_fade_width(&t);
    let Some(overflow) =
        crate::carousel::geometry::bounds(c.view.viewport, edges, c.view.vertical.get())
    else {
        return style();
    };
    if overflow.1 <= 1.0 {
        return style();
    }
    let start = c.looping || overflow.0 > 1.0;
    let end = c.looping || overflow.0 < overflow.1 - 1.0;
    let mask = kivact_core::style::linear_gradient(if c.view.vertical.get() { 180 } else { 90 })
        .stop(
            if start {
                kivact_core::style::Color::TRANSPARENT
            } else {
                kivact_core::style::Color::BLACK
            },
            0,
        )
        .stop(kivact_core::style::Color::BLACK, fade)
        .stop(kivact_core::style::Color::BLACK, pct(100) - fade)
        .stop(
            if end {
                kivact_core::style::Color::TRANSPARENT
            } else {
                kivact_core::style::Color::BLACK
            },
            pct(100),
        );
    style().mask(mask)
}

/// Route unmodified axis arrow keys through the same page navigation command.
fn keyboard(c: Layout, references: &[NodeRef], e: kivact_core::view::KeyEvent) -> Handled {
    let m = e.modifiers();
    if m.ctrl || m.alt || m.meta || m.shift {
        return Handled::No;
    }
    let rtl = c.view.rtl == ReadingDirection::Rtl;
    let delta = match (c.view.vertical.get_untracked(), e.key()) {
        (true, Key::ArrowDown) => 1,
        (true, Key::ArrowUp) => -1,
        (false, Key::ArrowRight) => {
            if rtl {
                -1
            } else {
                1
            }
        }
        (false, Key::ArrowLeft) => {
            if rtl {
                1
            } else {
                -1
            }
        }
        _ => return Handled::No,
    };
    navigate(c, references, delta);
    Handled::Yes
}

/// Render one previous/next action with the correct directional mark and bounds.
fn navigation_button(
    c: Layout,
    variants: &kivact_core::view::Variants,
    references: Rc<Vec<NodeRef>>,
    edges: (Option<NodeRef>, Option<NodeRef>),
    next: bool,
) -> View {
    let button = crate::button(if next {
        kivact_core::i18n::message("carousel.next", "Next slide")
    } else {
        kivact_core::i18n::message("carousel.previous", "Previous slide")
    })
    .hide_label(true)
    .icon(kivact_core::view::icon(move || {
        kivact_core::style::Icon::named(
            match (
                c.view.vertical.get(),
                c.view.rtl == ReadingDirection::Rtl,
                next,
            ) {
                (true, _, true) => "chevron-down",
                (true, _, false) => "chevron-up",
                (false, true, true) | (false, false, false) => "chevron-left",
                _ => "chevron-right",
            },
        )
    }))
    .hidden(move || !c.buttons.get())
    .disabled(move || {
        c.disabled.get()
            || if c.mode == CarouselMode::Scroll {
                !crate::carousel::geometry::can_scroll(
                    c.view.viewport,
                    edges,
                    c.view.vertical.get(),
                    c.view.rtl == ReadingDirection::Rtl,
                    next,
                    c.looping,
                )
            } else {
                c.last.get() == 0
                    || (!c.looping
                        && if next {
                            c.shown.get() >= c.last.get()
                        } else {
                            c.shown.get() == 0
                        })
            }
    })
    .on_press(move || navigate(c, &references, if next { 1 } else { -1 }));
    let pick: fn(&CarouselTheme) -> &crate::recipe::Recipe<CarouselSurfaceState> = if next {
        |r: &CarouselTheme| r.next_recipe()
    } else {
        |r: &CarouselTheme| r.previous_recipe()
    };
    crate::layer::with_recipe_style(button, pick, variants, move || c.recipe_state()).into_view()
}

/// The structural layout used by a carousel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselMode {
    #[default]
    Translate,
    Scroll,
}
