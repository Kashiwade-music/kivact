//! Role: Render binding-driven translated or native scrolling carousels.
use kivact_core::geometry::Point;
use kivact_core::state::{Binding, IntoBinding, IntoValue, Value, effect, memo};
use kivact_core::style::{Direction, Overflow, pct, style};
use kivact_core::view::{
    Handled, IntoChildren, IntoView, Key, Node, NodeBuilder, Orientation, ReadingDirection, Role,
    ScrollBehavior, View, node, node_ref, use_direction,
};
pub use kivact_interaction::Snap;
use kivact_interaction::use_carousel;
/// The structural layout used by a carousel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselMode {
    #[default]
    Translate,
    Scroll,
}
/// A slide viewport and previous/next controls.
pub struct Carousel {
    root: Node,
    slides: Vec<View>,
    index: Binding<usize>,
    orientation: Value<Orientation>,
    snap: Value<Snap>,
    mode: CarouselMode,
    looping: bool,
    per_view: Value<f64>,
    gap: Value<f64>,
    padding: Value<f64>,
    buttons: Value<bool>,
    edge_fade: Value<bool>,
}
/// Show slides in caller-owned index order. Arrow keys follow orientation and reading direction.
#[track_caller]
pub fn carousel(index: impl IntoBinding<usize>, slides: impl IntoChildren) -> Carousel {
    Carousel {
        root: node().name("Carousel"),
        slides: slides.into_views(),
        index: index.into_binding(),
        orientation: Orientation::Horizontal.into_value(),
        snap: Snap::Mandatory.into_value(),
        mode: CarouselMode::Translate,
        looping: false,
        per_view: 1.0.into_value(),
        gap: 16.0.into_value(),
        padding: 0.0.into_value(),
        buttons: true.into_value(),
        edge_fade: true.into_value(),
    }
}
impl Carousel {
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
        self.per_view = memo(move || {
            let n = count.get();
            if n.is_finite() { n.max(1.0) } else { 1.0 }
        });
        self
    }
    /// Space between slides in logical pixels.
    pub fn gap(mut self, gap: impl IntoValue<f64>) -> Self {
        self.gap = gap.into_value();
        self
    }
    /// Viewport inset in logical pixels.
    pub fn padding(mut self, padding: impl IntoValue<f64>) -> Self {
        self.padding = padding.into_value();
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
    fn into_view(self) -> View {
        let Self {
            root,
            slides,
            index,
            orientation,
            snap,
            mode,
            looping,
            per_view,
            gap,
            padding,
            buttons,
            edge_fade: _,
        } = self;
        let count = slides.len();
        let api = use_carousel(index, count);
        let vertical = memo(move || orientation.get() == Orientation::Vertical);
        let rtl = use_direction();
        let viewport = node_ref();
        let track = node_ref();
        let last = memo(move || count.saturating_sub(per_view.get().ceil() as usize));
        let shown = memo(move || api.index().get().min(last.get()));
        let go = move |delta: isize| {
            let at = shown.get_untracked() as isize;
            let period = last.get_untracked() as isize + 1;
            api.go_to(if looping {
                (at + delta).rem_euclid(period)
            } else {
                (at + delta).clamp(0, period - 1)
            } as usize);
        };
        let refs: Vec<_> = (0..count).map(|_| node_ref()).collect();
        let refs = std::rc::Rc::new(refs);
        if mode == CarouselMode::Scroll {
            let navigation_refs = refs.clone();
            effect(move || {
                let at = shown.get();
                if let (Some(view), Some(slide)) = (
                    viewport.rect().get(),
                    navigation_refs
                        .get(at)
                        .and_then(|r| r.rect().get_untracked()),
                ) {
                    let old = viewport.scroll_offset().get_untracked();
                    let point = if vertical.get() {
                        Point {
                            x: old.x,
                            y: old.y + slide.y - view.y - padding.get(),
                        }
                    } else {
                        Point {
                            x: old.x + slide.x - view.x - padding.get(),
                            y: old.y,
                        }
                    };
                    viewport.scroll_to(point, ScrollBehavior::Instant);
                }
            });
            let navigation_refs = refs.clone();
            effect(move || {
                let offset = viewport.scroll_offset().get();
                if let Some(view) = viewport.rect().get_untracked() {
                    let nearest = navigation_refs
                        .iter()
                        .enumerate()
                        .filter_map(|(i, r)| {
                            r.rect().get_untracked().map(|r| {
                                (
                                    i,
                                    if vertical.get_untracked() {
                                        (r.y - view.y).abs()
                                    } else {
                                        (r.x - view.x).abs()
                                    },
                                )
                            })
                        })
                        .min_by(|a, b| a.1.total_cmp(&b.1))
                        .map(|p| p.0);
                    if let Some(at) = nearest {
                        let _ = offset;
                        if snap.get() == Snap::Mandatory {
                            api.scrolling(at);
                        }
                    }
                }
            });
        }
        let items: Vec<_> = slides
            .into_iter()
            .enumerate()
            .map(|(at, item)| {
                node()
                    .node_ref(refs[at])
                    .with_node(|n| n.role(Role::Group))
                    .name(format!("Slide {} / {}", at + 1, count))
                    .style(move || {
                        style()
                            .grow(0)
                            .shrink(0)
                            .basis(pct(100.0 / per_view.get()))
                            .min_width(0)
                            .min_height(0)
                    })
                    .child(item)
                    .into_view()
            })
            .collect();
        let content = node()
            .node_ref(track)
            .style(move || {
                let mut s = style()
                    .direction(if vertical.get() {
                        Direction::Column
                    } else if rtl.get() == ReadingDirection::Rtl {
                        Direction::RowReverse
                    } else {
                        Direction::Row
                    })
                    .gap(gap.get());
                if mode == CarouselMode::Translate {
                    let along = -(shown.get() as f64) * 100.0 / per_view.get();
                    s = if vertical.get() {
                        s.translate(0, pct(along))
                    } else {
                        s.translate(
                            pct(if rtl.get() == ReadingDirection::Rtl {
                                -along
                            } else {
                                along
                            }),
                            0,
                        )
                    };
                }
                s
            })
            .child(items);
        let viewport_node = node()
            .node_ref(viewport)
            .style(move || {
                style()
                    .min_width(0)
                    .min_height(0)
                    .padding(padding.get())
                    .overflow(if mode == CarouselMode::Scroll {
                        Overflow::Scroll
                    } else {
                        Overflow::Hidden
                    })
            })
            .on_wheel(move |event| {
                if mode == CarouselMode::Scroll
                    && !vertical.get_untracked()
                    && event.modifiers().shift
                    && event.delta().x == 0.0
                    && event.delta().y != 0.0
                {
                    viewport.scroll_by(
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
            .child(content);
        let previous = crate::button("Previous slide")
            .hidden(move || !buttons.get())
            .disabled(move || !looping && shown.get() == 0)
            .on_press(move || go(-1));
        let next = crate::button("Next slide")
            .hidden(move || !buttons.get())
            .disabled(move || !looping && shown.get() >= last.get())
            .on_press(move || go(1));
        root.with_node(|n| n.role(Role::Group))
            .focusable(true)
            .on_key(move |e| {
                let m = e.modifiers();
                if m.ctrl || m.alt || m.meta || m.shift {
                    return Handled::No;
                }
                let delta = match (vertical.get_untracked(), e.key()) {
                    (true, Key::ArrowDown) => 1,
                    (true, Key::ArrowUp) => -1,
                    (false, Key::ArrowRight) => {
                        if rtl.get_untracked() == ReadingDirection::Rtl {
                            -1
                        } else {
                            1
                        }
                    }
                    (false, Key::ArrowLeft) => {
                        if rtl.get_untracked() == ReadingDirection::Rtl {
                            1
                        } else {
                            -1
                        }
                    }
                    _ => return Handled::No,
                };
                go(delta);
                Handled::Yes
            })
            .child(viewport_node)
            .child((previous, next))
            .into_view()
    }
}
