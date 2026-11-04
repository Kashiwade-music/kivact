//! Role: The Lyra carousel with previous and next buttons and animated slides.

use kivact::i18n::message;
use kivact::motion::{Easing, Transition};
use kivact::state::IntoBinding;
use kivact::state::IntoValue;
use kivact::state::{Binding, Value, memo};
use kivact::style::{Direction, Overflow, Position, TransitionProp, pct, style};
use kivact::ui::{ButtonVariant, ControlSize, button};
use kivact::view::IntoChildren;
use kivact::view::{Handled, IntoView, Key, Orientation, Role, View};
use kivact::view::{Node, NodeBuilder, icon, node};
use kivact::view::{ReadingDirection, use_direction};

/// A row or column of slides with step buttons (carousel.tsx). Created by [`carousel`].
pub struct Carousel {
    root: Node,
    items: Vec<View>,
    index: Binding<usize>,
    orientation: Value<Orientation>,
    per_view: Value<f64>,
    looped: Value<bool>,
}

/// A carousel of `items`, a tuple of views or a `Vec` of them, showing the slide
/// `index` (zero-based) first.
///
/// The index is the binding's value: the buttons, the arrow keys (Left and
/// Right, or Up and Down for a vertical carousel; mirrored in a right-to-left
/// [`direction`](kivact::view::provide_direction)) and the application all move it, and
/// the slides slide to it. The root takes the focus so the keys work, and the
/// buttons (outline icon buttons named "Previous slide" and "Next slide")
/// sit outside its box, 48 px away, so leave room. With
/// [`Carousel::items_per_view`] above 1 the last position is the one that
/// still shows a full set.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn slides() -> impl IntoView {
///     let index = signal(0usize);
///     basic::carousel(
///         (1..=5)
///             .map(|n| text(format!("Slide {n}")))
///             .collect::<Vec<_>>(),
///         index,
///     )
///     .style(style().width(240).margin_x(48))
/// }
/// ```
#[track_caller]
pub fn carousel(
    items: impl IntoChildren,
    index: impl kivact::state::IntoBinding<usize>,
) -> Carousel {
    Carousel {
        root: node()
            .with_node(|root| root.role(Role::Group))
            .name(message("basic.carousel.label", "Carousel"))
            .focusable(true)
            .style(move || {
                // The theme's ring whenever the region has the focus.
                style()
                    .position(Position::Relative)
                    .direction(Direction::Column)
                    .focus(crate::theme_access::theme_ring())
            }),
        items: items.into_views(),
        index: index.into_binding(),
        orientation: Orientation::Horizontal.into_value(),
        per_view: 1.0.into_value(),
        looped: false.into_value(),
    }
}

impl Carousel {
    /// Choose the direction slides move in: [`Orientation::Horizontal`] puts the
    /// slides side by side with the buttons left and right of the carousel,
    /// [`Orientation::Vertical`] stacks them with the buttons above and below
    /// (`Unspecified` is horizontal). A constant: it is the axis of the track,
    /// of the swipe and of the arrow keys, fixed when the carousel is built.
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        self.orientation = orientation.into_value();
        self
    }

    /// Show this many slides at once (1 by default; fractions show a peek of the next slide).
    pub fn items_per_view(mut self, count: impl IntoValue<f64>) -> Self {
        let count = count.into_value();
        self.per_view = (move || {
            let count = count.get();
            if count.is_finite() && count >= 1.0 {
                count
            } else {
                1.0
            }
        })
        .into_value();
        self
    }

    /// Wrap from the last position to the first and back while true
    /// (constant or reactive).
    #[track_caller]
    pub fn looping(mut self) -> Self {
        self.looped = true.into_value();
        self
    }
}

impl NodeBuilder for Carousel {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

const GAP: f64 = 16.0;

impl IntoView for Carousel {
    fn into_view(self) -> View {
        let Self {
            root,
            items,
            index,
            orientation,
            per_view,
            looped,
        } = self;
        let rtl = use_direction().get() == ReadingDirection::Rtl;
        let vertical = memo(move || orientation.get() == Orientation::Vertical);
        let count = items.len();
        let last = memo(move || count.saturating_sub(per_view.get().ceil() as usize));
        let value = index.into_value();
        let shown = memo(move || value.get().min(last.get()));
        // Steps in the order of reading: +1 is toward the next slide.
        let go = move |delta: isize| {
            let at = shown.get_untracked() as isize;
            let period = last.get() as isize + 1;
            let target = if looped.get_untracked() {
                (at + delta).rem_euclid(period)
            } else {
                (at + delta).clamp(0, last.get() as isize)
            };
            if target != at {
                index.set(target as usize);
            }
        };
        let can_prev = memo(move || last.get() > 0 && (looped.get() || shown.get() > 0));
        let can_next = memo(move || last.get() > 0 && (looped.get() || shown.get() < last.get()));

        let slide_label = message("basic.carousel.slide", "Slide");
        let slides: Vec<View> = items
            .into_iter()
            .enumerate()
            .map(|(at, item)| {
                let name = memo(move || format!("{} {} / {}", slide_label.get(), at + 1, count));
                node()
                    .with_node(|root| root.role(Role::Group))
                    .name(name)
                    .style(move || {
                        let s = style()
                            .grow(0)
                            .shrink(0)
                            .basis(pct(100.0 / per_view.get()))
                            .min_width(0)
                            .min_height(0);
                        if vertical.get() {
                            s.padding_top(GAP)
                        } else {
                            s.padding_left(GAP)
                        }
                    })
                    .child(item)
                    .into_view()
            })
            .collect();
        let track = node()
            .style(move || {
                // The track is one gap wider than the viewport, so each slide
                // (its basis is a share of the track) moves by its own width.
                let along = -(shown.get() as f64) * 100.0 / per_view.get();
                let s = style()
                    .direction(match (vertical.get(), rtl) {
                        (true, _) => Direction::Column,
                        (false, true) => Direction::RowReverse,
                        (false, false) => Direction::Row,
                    })
                    .transition(
                        TransitionProp::Paint,
                        Transition::tween(300, Easing::CubicBezier(0.4, 0.0, 0.2, 1.0)),
                    );
                if vertical.get() {
                    s.margin_top(-GAP).translate(0, pct(along))
                } else if rtl {
                    s.margin_right(-GAP).translate(pct(-along), 0)
                } else {
                    s.margin_left(-GAP).translate(pct(along), 0)
                }
            })
            .child(slides);
        let viewport = node()
            .style(
                style()
                    .direction(Direction::Column)
                    .overflow(Overflow::Hidden),
            )
            .child(track);

        let previous = button(message("basic.carousel.previous", "Previous slide"))
            .variant(ButtonVariant::Outline)
            .hide_label(true)
            .icon(icon(move || {
                kivact::style::Icon::named(if vertical.get() || !rtl {
                    if vertical.get() {
                        "chevron-up"
                    } else {
                        "chevron-left"
                    }
                } else {
                    "chevron-right"
                })
            }))
            .size(ControlSize::Small)
            .disabled(move || !can_prev.get())
            .on_press(move || go(-1))
            .style(move || {
                let s = style().position(Position::Absolute);
                match (vertical.get(), rtl) {
                    (true, _) => s.top(-48).left(pct(50) - 14),
                    (false, false) => s.left(-48).top(pct(50) - 14),
                    (false, true) => s.right(-48).top(pct(50) - 14),
                }
            });
        let next = button(message("basic.carousel.next", "Next slide"))
            .variant(ButtonVariant::Outline)
            .hide_label(true)
            .icon(icon(move || {
                kivact::style::Icon::named(if vertical.get() {
                    "chevron-down"
                } else if rtl {
                    "chevron-left"
                } else {
                    "chevron-right"
                })
            }))
            .size(ControlSize::Small)
            .disabled(move || !can_next.get())
            .on_press(move || go(1))
            .style(move || {
                let s = style().position(Position::Absolute);
                match (vertical.get(), rtl) {
                    (true, _) => s.bottom(-48).left(pct(50) - 14),
                    (false, false) => s.right(-48).top(pct(50) - 14),
                    (false, true) => s.left(-48).top(pct(50) - 14),
                }
            });

        root.on_key(move |event| {
            let m = event.modifiers();
            if m.ctrl || m.alt || m.meta || m.shift {
                return Handled::No;
            }
            let step = match (vertical.get(), event.key()) {
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
                (true, Key::ArrowDown) => 1,
                (true, Key::ArrowUp) => -1,
                _ => return Handled::No,
            };
            go(step);
            Handled::Yes
        })
        .child(viewport)
        .child(previous)
        .child(next)
        .into_view()
    }
}
