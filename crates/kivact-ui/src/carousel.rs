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
    fn into_view(self) -> View {
        let Self {
            mut root,
            variants,
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
            edge_fade,
        } = self;
        let disabled = root.accessible_disabled();
        let disabled = memo(move || disabled.is_some_and(|d| d.get()));
        let count = slides.len();
        let api = use_carousel(index, count);
        let state = api.state();
        let padding_value = move || {
            padding.map_or_else(
                || crate::look_files::carousel_padding(&kivact_core::style::use_theme().get()),
                |p| p.get(),
            )
        };
        let gap_value = move || {
            gap.map_or_else(
                || crate::look_files::carousel_gap(&kivact_core::style::use_theme().get()),
                |p| p.get(),
            )
        };
        let vertical = memo(move || orientation.get() == Orientation::Vertical);
        let rtl = use_direction();
        let viewport = node_ref();
        let track = node_ref();
        let visible = move || per_view.map_or(1.0, |n| n.get());
        let last = memo(move || count.saturating_sub(visible().ceil() as usize));
        let shown = memo(move || api.index().get().min(last.get()));
        let refs: Vec<_> = (0..count).map(|_| node_ref()).collect();
        let refs = std::rc::Rc::new(refs);
        let navigation_refs = refs.clone();
        let go = move |delta: isize| {
            if disabled.get_untracked() {
                return;
            }
            if mode == CarouselMode::Scroll {
                if let Some(target) = crate::carousel_geometry::page(
                    viewport,
                    track,
                    &navigation_refs,
                    vertical.get_untracked(),
                    rtl.get_untracked() == ReadingDirection::Rtl,
                    delta > 0,
                    looping,
                    snap.get_untracked(),
                    padding_value(),
                ) {
                    api.go_to(target);
                }
                return;
            }
            let at = shown.get_untracked() as isize;
            let period = last.get_untracked() as isize + 1;
            api.go_to(if looping {
                (at + delta).rem_euclid(period)
            } else {
                (at + delta).clamp(0, period - 1)
            } as usize);
        };
        if mode == CarouselMode::Scroll {
            let navigation_refs = refs.clone();
            effect(move || {
                let at = shown.get();
                if !matches!(state.get(), kivact_interaction::CarouselState::Idle { .. }) {
                    return;
                }
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
                            y: old.y + slide.y - view.y - padding_value(),
                        }
                    } else {
                        Point {
                            x: old.x
                                + if rtl.get() == ReadingDirection::Rtl {
                                    slide.right() - view.right() + padding_value()
                                } else {
                                    slide.x - view.x - padding_value()
                                },
                            y: old.y,
                        }
                    };
                    if snap.get() == Snap::None {
                        let nearest = navigation_refs
                            .iter()
                            .enumerate()
                            .filter_map(|(i, r)| {
                                r.rect().get_untracked().map(|r| {
                                    (
                                        i,
                                        if vertical.get() {
                                            (r.y - view.y).abs()
                                        } else {
                                            if rtl.get() == ReadingDirection::Rtl {
                                                (r.right() - view.right()).abs()
                                            } else {
                                                (r.x - view.x).abs()
                                            }
                                        },
                                    )
                                })
                            })
                            .min_by(|a, b| a.1.total_cmp(&b.1))
                            .map(|p| p.0);
                        if nearest == Some(at) {
                            return;
                        }
                    }
                    viewport.scroll_to(
                        point,
                        if kivact_core::platform::is_motion_reduced().get_untracked() {
                            ScrollBehavior::Instant
                        } else {
                            ScrollBehavior::Smooth(kivact_core::motion::Easing::EaseOut)
                        },
                    );
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
                                        if rtl.get() == ReadingDirection::Rtl {
                                            (r.right() - view.right()).abs()
                                        } else {
                                            (r.x - view.x).abs()
                                        }
                                    },
                                )
                            })
                        })
                        .min_by(|a, b| a.1.total_cmp(&b.1))
                        .map(|p| p.0);
                    if let Some(at) = nearest {
                        let _ = offset;
                        api.scrolling(at);
                    }
                }
            });
        }
        let items: Vec<_> = slides
            .into_iter()
            .enumerate()
            .map(|(at, item)| {
                crate::layer::with_recipe_style(
                    node(),
                    |r| r.carousel_recipes().slide_recipe(),
                    &variants,
                    move || state.get(),
                )
                .node_ref(refs[at])
                .with_node(|n| n.role(Role::Group))
                .name(format!("Slide {} / {}", at + 1, count))
                .style(move || {
                    style()
                        .grow(0)
                        .shrink(0)
                        .basis(if mode == CarouselMode::Translate || per_view.is_some() {
                            pct(100.0 / visible()) - gap_value() * (visible() - 1.0) / visible()
                        } else {
                            kivact_core::style::Length::Auto
                        })
                        .min_width(0)
                        .min_height(0)
                })
                .child(item)
                .into_view()
            })
            .collect();
        let content = crate::layer::with_recipe_style(
            node(),
            |r| r.carousel_recipes().track_recipe(),
            &variants,
            move || state.get(),
        )
        .node_ref(track)
        .style(move || {
            let mut s = style().direction(if vertical.get() {
                Direction::Column
            } else if rtl.get() == ReadingDirection::Rtl {
                Direction::RowReverse
            } else {
                Direction::Row
            });
            if let Some(gap) = gap {
                s = s.gap(gap.get());
            }
            if mode == CarouselMode::Translate {
                let along = -(shown.get() as f64) * 100.0 / visible();
                let offset = pct(along) - (shown.get() as f64) * gap_value() / visible();
                s = if vertical.get() {
                    s.translate(0, offset)
                } else {
                    s.translate(
                        if rtl.get() == ReadingDirection::Rtl {
                            pct(-along) + (shown.get() as f64) * gap_value() / visible()
                        } else {
                            offset
                        },
                        0,
                    )
                };
            }
            s
        })
        .child(items);
        let viewport_node = crate::layer::with_recipe_style(
            node(),
            |r| r.carousel_recipes().viewport_recipe(),
            &variants,
            move || state.get(),
        )
        .node_ref(viewport)
        .style(move || {
            style()
                .min_width(0)
                .min_height(0)
                .padding(padding_value())
                .overflow(if mode == CarouselMode::Scroll {
                    Overflow::Scroll
                } else {
                    Overflow::Hidden
                })
        })
        .style(move || {
            if !edge_fade.get() || mode != CarouselMode::Scroll {
                return style();
            }
            let t = kivact_core::style::use_theme().get();
            let fade = crate::look_files::carousel_fade_width(&t);
            let offset = viewport.scroll_offset().get();
            let overflow = match (viewport.rect().get(), track.rect().get()) {
                (Some(v), Some(t)) => {
                    if vertical.get() {
                        (offset.y, (t.height - v.height).max(0.0))
                    } else {
                        (offset.x, (t.width - v.width).max(0.0))
                    }
                }
                _ => return style(),
            };
            if overflow.1 <= 1.0 {
                return style();
            }
            let start = looping || overflow.0 > 1.0;
            let end = looping || overflow.0 < overflow.1 - 1.0;
            let mask = kivact_core::draw::linear_gradient(if vertical.get() { 180 } else { 90 })
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
        })
        .on_wheel(move |event| {
            if !disabled.get_untracked()
                && mode == CarouselMode::Scroll
                && !vertical.get_untracked()
                && event.modifiers().shift
                && event.delta().x == 0.0
                && event.delta().y != 0.0
                && (crate::carousel_geometry::can_scroll(viewport, track, false, false, true, true))
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
        let previous_go = go.clone();
        let next_go = go.clone();
        let previous = crate::button("Previous slide")
            .hidden(move || !buttons.get())
            .disabled(move || {
                disabled.get()
                    || if mode == CarouselMode::Scroll {
                        !crate::carousel_geometry::can_scroll(
                            viewport,
                            track,
                            vertical.get(),
                            rtl.get() == ReadingDirection::Rtl,
                            false,
                            looping,
                        )
                    } else {
                        last.get() == 0 || (!looping && shown.get() == 0)
                    }
            })
            .on_press(move || previous_go(-1));
        let next = crate::button("Next slide")
            .hidden(move || !buttons.get())
            .disabled(move || {
                disabled.get()
                    || if mode == CarouselMode::Scroll {
                        !crate::carousel_geometry::can_scroll(
                            viewport,
                            track,
                            vertical.get(),
                            rtl.get() == ReadingDirection::Rtl,
                            true,
                            looping,
                        )
                    } else {
                        last.get() == 0 || (!looping && shown.get() >= last.get())
                    }
            })
            .on_press(move || next_go(1));
        let previous = crate::layer::with_recipe_style(
            previous,
            |r| r.carousel_recipes().previous_recipe(),
            &variants,
            move || state.get(),
        );
        let next = crate::layer::with_recipe_style(
            next,
            |r| r.carousel_recipes().next_recipe(),
            &variants,
            move || state.get(),
        );
        crate::layer::with_base_recipe_style(
            root,
            |r| r.carousel_recipes().root_recipe(),
            &variants,
            move || state.get(),
        )
        .with_node(|n| n.role(Role::Group))
        .focusable(move || !disabled.get())
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
