//! Role: Render binding-driven translated or native scrolling carousels.
use kivact_core::geometry::Point;
use kivact_core::state::{Binding, IntoBinding, IntoValue, Value, effect, memo};
use kivact_core::style::{Direction, Overflow, pct, style};
use kivact_core::view::{
    Handled, IntoChildren, IntoView, Key, Node, NodeBuilder, Orientation, ReadingDirection, Role,
    ScrollBehavior, View, node, node_ref, use_direction,
};
pub use kivact_interaction::Snap;
/// Passive navigation and layout facts supplied to carousel recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct CarouselSurfaceState {
    pub state: kivact_interaction::CarouselState,
    pub orientation: Orientation,
    pub direction: ReadingDirection,
}
impl std::ops::Deref for CarouselSurfaceState {
    type Target = kivact_interaction::CarouselState;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

use kivact_interaction::use_carousel;
/// The structural layout used by a carousel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselMode {
    #[default]
    Translate,
    Scroll,
}
// xtask-allow(state-fields): Independent setup-only NodeBuilder options; navigation and settling live in CarouselState.
/// A slide viewport and previous/next controls.
pub struct Carousel {
    root: Node,
    variants: crate::variants::Variants,
    slides: Vec<View>,
    index: Binding<usize>,
    orientation: Value<Orientation>,
    snap: Value<Snap>,
    mode: CarouselMode,
    looping: bool,
    per_view: Option<Value<f64>>,
    gap: Option<Value<f64>>,
    padding: Option<Value<f64>>,
    buttons: Value<bool>,
    edge_fade: Value<bool>,
}
/// Show slides in caller-owned index order. Arrow keys follow orientation and reading direction.
#[track_caller]
pub fn carousel(index: impl IntoBinding<usize>, slides: impl IntoChildren) -> Carousel {
    Carousel {
        variants: crate::variants::Variants::default(),
        root: node().name("Carousel"),
        slides: slides.into_views(),
        index: index.into_binding(),
        orientation: Orientation::Horizontal.into_value(),
        snap: Snap::Mandatory.into_value(),
        mode: CarouselMode::Translate,
        looping: false,
        per_view: None,
        gap: None,
        padding: None,
        buttons: true.into_value(),
        edge_fade: true.into_value(),
    }
}
impl Carousel {
    /// Add a setup-only visual variant for every component recipe.
    pub fn variant(self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }

    /// Wrap previous and next navigation at the endpoints.
    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }
    /// Choose scroll settling behavior.
    pub fn snap(mut self, snap: impl IntoValue<Snap>) -> Self {
        self.snap = snap.into_value();
        self
    }
    /// Choose the slide axis.
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        self.orientation = orientation.into_value();
        self
    }
    /// Choose translated slides or a native scrolling viewport. Setup only.
    pub fn mode(mut self, mode: CarouselMode) -> Self {
        self.mode = mode;
        self
    }
    /// Number of slides visible, including fractional peeks.
    pub fn items_per_view(mut self, count: impl IntoValue<f64>) -> Self {
        let count = count.into_value();
        self.per_view = Some(memo(move || {
            let n = count.get();
            if n.is_finite() { n.max(1.0) } else { 1.0 }
        }));
        self
    }
    /// Space between slides in logical pixels.
    pub fn gap(mut self, gap: impl IntoValue<f64>) -> Self {
        self.gap = Some(gap.into_value());
        self
    }
    /// Viewport inset in logical pixels.
    pub fn padding(mut self, padding: impl IntoValue<f64>) -> Self {
        self.padding = Some(padding.into_value());
        self
    }
    /// Show previous and next controls.
    pub fn buttons(mut self, buttons: impl IntoValue<bool>) -> Self {
        self.buttons = buttons.into_value();
        self
    }
    /// Fade overflowing viewport edges.
    pub fn edge_fade(mut self, fade: impl IntoValue<bool>) -> Self {
        self.edge_fade = fade.into_value();
        self
    }
}
impl NodeBuilder for Carousel {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Carousel {
    fn into_view(mut self) -> View {
        let c = prepare(&mut self);
        let references = std::rc::Rc::new((0..c.count).map(|_| node_ref()).collect::<Vec<_>>());
        let edges = (references.first().copied(), references.last().copied());
        if c.mode == CarouselMode::Scroll {
            follow_index(c, references.clone());
            follow_scroll(c, references.clone());
        }
        let items = slide_items(c, &self.variants, self.slides, &references);
        let content = slide_track(c, &self.variants, items);
        let viewport = slide_viewport(c, &self.variants, content, edges);
        let previous = navigation_button(c, &self.variants, references.clone(), edges, false);
        let next = navigation_button(c, &self.variants, references.clone(), edges, true);
        crate::layer::with_base_recipe_style(
            self.root,
            |r| r.carousel_recipes().root_recipe(),
            &self.variants,
            move || c.recipe_state(),
        )
        .with_node(|n| n.role(Role::Group))
        .focusable(move || !c.disabled.get())
        .on_key(move |event| keyboard(c, &references, event))
        .child(viewport)
        .child((previous, next))
        .into_view()
    }
}
// xtask-allow(state-fields): Immutable construction settings and actual layout/interaction handles; CarouselApi owns all navigation and settling state.
#[derive(Clone, Copy)]
struct Layout {
    api: kivact_interaction::CarouselApi,
    mode: CarouselMode,
    looping: bool,
    snap: Value<Snap>,
    vertical: Value<bool>,
    orientation: Value<Orientation>,
    rtl: Value<ReadingDirection>,
    viewport: kivact_core::view::NodeRef,
    track: kivact_core::view::NodeRef,
    count: usize,
    last: Value<usize>,
    shown: Value<usize>,
    disabled: Value<bool>,
    buttons: Value<bool>,
    edge_fade: Value<bool>,
    per_view: Option<Value<f64>>,
    gap: Option<Value<f64>>,
    padding: Option<Value<f64>>,
}
impl Layout {
    fn padding(self) -> f64 {
        self.padding.map_or_else(
            || crate::look_files::carousel_padding(&kivact_core::style::use_theme().get()),
            |v| v.get(),
        )
    }
    fn gap(self) -> f64 {
        self.gap.map_or_else(
            || crate::look_files::carousel_gap(&kivact_core::style::use_theme().get()),
            |v| v.get(),
        )
    }
    fn visible(self) -> f64 {
        self.per_view.map_or(1.0, |v| v.get())
    }
    fn recipe_state(self) -> CarouselSurfaceState {
        CarouselSurfaceState {
            state: self.api.state().get(),
            orientation: self.orientation.get(),
            direction: self.rtl.get(),
        }
    }
}
/// Prepare immutable layout inputs and authoritative navigation handles.
fn prepare(carousel: &mut Carousel) -> Layout {
    let disabled = carousel.root.accessible_disabled();
    let disabled = memo(move || disabled.is_some_and(|v| v.get()));
    let count = carousel.slides.len();
    let api = use_carousel(carousel.index, count);
    let orientation = carousel.orientation;
    let vertical = memo(move || orientation.get() == Orientation::Vertical);
    let per_view = carousel.per_view;
    let last =
        memo(move || count.saturating_sub(per_view.map_or(1.0, |v| v.get()).ceil() as usize));
    let shown = memo(move || api.index().get().min(last.get()));
    Layout {
        api,
        mode: carousel.mode,
        looping: carousel.looping,
        snap: carousel.snap,
        vertical,
        orientation,
        rtl: use_direction(),
        viewport: node_ref(),
        track: node_ref(),
        count,
        last,
        shown,
        disabled,
        buttons: carousel.buttons,
        edge_fade: carousel.edge_fade,
        per_view,
        gap: carousel.gap,
        padding: carousel.padding,
    }
}
/// Navigate one page using measured scrolling or the translated slide index.
fn navigate(c: Layout, references: &[kivact_core::view::NodeRef], delta: isize) {
    if c.disabled.get_untracked() {
        return;
    }
    if c.mode == CarouselMode::Scroll {
        if let Some(target) = crate::carousel_geometry::page(
            c.viewport,
            references,
            crate::carousel_geometry::PageRequest {
                vertical: c.vertical.get_untracked(),
                rtl: c.rtl.get_untracked() == ReadingDirection::Rtl,
                next: delta > 0,
                looping: c.looping,
                snap: c.snap.get_untracked(),
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
/// Scroll the selected slide into view when the interaction authority is idle.
fn follow_index(c: Layout, navigation_refs: std::rc::Rc<Vec<kivact_core::view::NodeRef>>) {
    effect(move || {
        let at = c.shown.get();
        if !matches!(
            c.api.state().get(),
            kivact_interaction::CarouselState::Idle { .. }
        ) {
            return;
        }
        if let (Some(view), Some(slide)) = (
            c.viewport.rect().get(),
            navigation_refs
                .get(at)
                .and_then(|r| r.rect().get_untracked()),
        ) {
            let old = c.viewport.scroll_offset().get_untracked();
            let point = if c.vertical.get() {
                Point {
                    x: old.x,
                    y: old.y + slide.y - view.y - c.padding(),
                }
            } else {
                Point {
                    x: old.x
                        + if c.rtl.get() == ReadingDirection::Rtl {
                            slide.right() - view.right() + c.padding()
                        } else {
                            slide.x - view.x - c.padding()
                        },
                    y: old.y,
                }
            };
            if c.snap.get() == Snap::None {
                let nearest = nearest_slide(
                    &navigation_refs,
                    view,
                    c.vertical.get(),
                    c.rtl.get() == ReadingDirection::Rtl,
                );
                if nearest == Some(at) {
                    return;
                }
            }
            c.viewport.scroll_to(
                point,
                if kivact_core::platform::is_motion_reduced().get_untracked() {
                    ScrollBehavior::Instant
                } else {
                    ScrollBehavior::Smooth(kivact_core::motion::Easing::EaseOut)
                },
            );
        }
    });
}
/// Feed the actual native scroll position back into the carousel authority.
fn follow_scroll(c: Layout, navigation_refs: std::rc::Rc<Vec<kivact_core::view::NodeRef>>) {
    effect(move || {
        let offset = c.viewport.scroll_offset().get();
        if let Some(view) = c.viewport.rect().get_untracked() {
            let nearest = nearest_slide(
                &navigation_refs,
                view,
                c.vertical.get_untracked(),
                c.rtl.get() == ReadingDirection::Rtl,
            );
            if let Some(at) = nearest {
                let _ = offset;
                c.api.scrolling(at);
            }
        }
    });
}
/// Locate the closest measured slide along the actual reading axis.
fn nearest_slide(
    references: &[kivact_core::view::NodeRef],
    view: kivact_core::geometry::Rect,
    vertical: bool,
    rtl: bool,
) -> Option<usize> {
    references
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            r.rect().get_untracked().map(|r| {
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
    variants: &crate::variants::Variants,
    slides: Vec<View>,
    refs: &[kivact_core::view::NodeRef],
) -> Vec<View> {
    slides
        .into_iter()
        .enumerate()
        .map(|(at, item)| {
            crate::layer::with_recipe_style(
                node(),
                |r| r.carousel_recipes().slide_recipe(),
                variants,
                move || c.recipe_state(),
            )
            .node_ref(refs[at])
            .with_node(|n| n.role(Role::Group))
            .name(format!("Slide {} / {}", at + 1, c.count))
            .style(move || {
                style()
                    .grow(0)
                    .shrink(0)
                    .basis(
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
fn slide_track(c: Layout, variants: &crate::variants::Variants, items: Vec<View>) -> Node {
    crate::layer::with_recipe_style(
        node(),
        |r| r.carousel_recipes().track_recipe(),
        variants,
        move || c.recipe_state(),
    )
    .node_ref(c.track)
    .style(move || {
        let mut s = style().direction(if c.vertical.get() {
            Direction::Column
        } else if c.rtl.get() == ReadingDirection::Rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        });
        if let Some(gap) = c.gap {
            s = s.gap(gap.get());
        }
        if c.mode == CarouselMode::Translate {
            let along = -(c.shown.get() as f64) * 100.0 / c.visible();
            let offset = pct(along) - (c.shown.get() as f64) * c.gap() / c.visible();
            s = if c.vertical.get() {
                s.translate(0, offset)
            } else {
                s.translate(
                    if c.rtl.get() == ReadingDirection::Rtl {
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
    variants: &crate::variants::Variants,
    content: Node,
    edges: (
        Option<kivact_core::view::NodeRef>,
        Option<kivact_core::view::NodeRef>,
    ),
) -> Node {
    crate::layer::with_recipe_style(
        node(),
        |r| r.carousel_recipes().viewport_recipe(),
        variants,
        move || c.recipe_state(),
    )
    .node_ref(c.viewport)
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
            && !c.vertical.get_untracked()
            && event.modifiers().shift
            && event.delta().x == 0.0
            && event.delta().y != 0.0
            && (crate::carousel_geometry::can_scroll(c.viewport, edges, false, false, true, true))
        {
            c.viewport.scroll_by(
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
fn edge_mask(
    c: Layout,
    edges: (
        Option<kivact_core::view::NodeRef>,
        Option<kivact_core::view::NodeRef>,
    ),
) -> kivact_core::style::Style {
    if !c.edge_fade.get() || c.mode != CarouselMode::Scroll {
        return style();
    }
    let t = kivact_core::style::use_theme().get();
    let fade = crate::look_files::carousel_fade_width(&t);
    let Some(overflow) = crate::carousel_geometry::bounds(c.viewport, edges, c.vertical.get())
    else {
        return style();
    };
    if overflow.1 <= 1.0 {
        return style();
    }
    let start = c.looping || overflow.0 > 1.0;
    let end = c.looping || overflow.0 < overflow.1 - 1.0;
    let mask = kivact_core::draw::linear_gradient(if c.vertical.get() { 180 } else { 90 })
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
fn keyboard(
    c: Layout,
    references: &[kivact_core::view::NodeRef],
    e: kivact_core::view::KeyEvent,
) -> Handled {
    let m = e.modifiers();
    if m.ctrl || m.alt || m.meta || m.shift {
        return Handled::No;
    }
    let delta = match (c.vertical.get_untracked(), e.key()) {
        (true, Key::ArrowDown) => 1,
        (true, Key::ArrowUp) => -1,
        (false, Key::ArrowRight) => {
            if c.rtl.get_untracked() == ReadingDirection::Rtl {
                -1
            } else {
                1
            }
        }
        (false, Key::ArrowLeft) => {
            if c.rtl.get_untracked() == ReadingDirection::Rtl {
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
    variants: &crate::variants::Variants,
    references: std::rc::Rc<Vec<kivact_core::view::NodeRef>>,
    edges: (
        Option<kivact_core::view::NodeRef>,
        Option<kivact_core::view::NodeRef>,
    ),
    next: bool,
) -> View {
    let button = crate::button(if next { "Next slide" } else { "Previous slide" })
        .hide_label(true)
        .icon(kivact_core::view::icon(move || {
            kivact_core::style::Icon::named(
                match (c.vertical.get(), c.rtl.get() == ReadingDirection::Rtl, next) {
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
                    !crate::carousel_geometry::can_scroll(
                        c.viewport,
                        edges,
                        c.vertical.get(),
                        c.rtl.get() == ReadingDirection::Rtl,
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
    let pick: fn(&crate::recipes::UiRecipes) -> &crate::recipes::Recipe<CarouselSurfaceState> =
        if next {
            |r| r.carousel_recipes().next_recipe()
        } else {
            |r| r.carousel_recipes().previous_recipe()
        };
    crate::layer::with_recipe_style(button, pick, variants, move || c.recipe_state()).into_view()
}
