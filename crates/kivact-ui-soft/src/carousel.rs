//! Role: Render a horizontal Neutral carousel with measured scroll controls.
use crate::scrollable_area::{ScrollAxis, use_scroll_overflow};
use crate::stack::{Space, rtl};
use crate::tokens;
use kivact::control::{Node, NodeBuilder, icon, node, row};
use kivact::draw::{Point, linear_gradient};
use kivact::motion::Transition;
use kivact::primitive::NodeExt;
use kivact::style::{
    Align, Color, Direction, Length, Position, ScrollbarWidth, TransitionProp, pct, rgba, style,
};
use kivact::task::{sleep, spawn};
use kivact::ui::{ButtonVariant, ControlSize, button};
use kivact::view::{
    Handled, IntoText, IntoView, NodeRef, Role, ScrollBehavior, View, WheelEvent, node_ref, provide,
};
use kivact::{Signal, effect_on, on_cleanup, signal};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

const SNAP_IDLE: Duration = Duration::from_millis(120);

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Options {
    looping: bool,
    snap: bool,
}

/// Measured carousel controls that can be shared with a caller.
#[derive(Clone, Copy)]
pub struct CarouselHandle {
    viewport: NodeRef,
    content: NodeRef,
    slides: Signal<Rc<[NodeRef]>>,
    options: Signal<Options>,
    padding: Signal<Space>,
}

pub fn use_carousel() -> CarouselHandle {
    CarouselHandle {
        viewport: node_ref(),
        content: node_ref(),
        slides: signal(Rc::from([])),
        options: signal(Options::default()),
        padding: signal(Space::S0),
    }
}

impl CarouselHandle {
    fn boundaries(self) -> (f32, f32) {
        let width = self.viewport.rect().get().map_or(0.0, |rect| rect.width);
        let content = self.content.rect().get().map_or(0.0, |rect| rect.width);
        (
            self.viewport.scroll_offset().get().x,
            (content - width).max(0.0),
        )
    }

    pub fn can_scroll_next(self) -> bool {
        self.can_scroll(true)
    }
    pub fn can_scroll_prev(self) -> bool {
        self.can_scroll(false)
    }
    pub fn scroll_next(self) {
        self.step(true);
    }
    pub fn scroll_prev(self) {
        self.step(false);
    }

    fn can_scroll(self, next: bool) -> bool {
        let (offset, limit) = self.boundaries();
        if limit <= 1.0 {
            return false;
        }
        if self.options.get().looping {
            return true;
        }
        if next != rtl() {
            offset < limit - 1.0
        } else {
            offset > 1.0
        }
    }

    pub fn scroll_to(self, index: usize) {
        if let Some(offset) = self.slide_offset(index) {
            self.go_to(offset);
        }
    }

    fn go_to(self, x: f32) {
        let motion = if kivact::platform::reduced_motion() {
            ScrollBehavior::Instant
        } else {
            ScrollBehavior::Smooth
        };
        self.viewport.scroll_to(Point { x, y: 0.0 }, motion);
    }

    fn slide_offset(self, index: usize) -> Option<f32> {
        let viewport = self.viewport.rect().get()?;
        let slides = self.slides.get();
        let rect = slides.get(index)?.rect().get()?;
        let padding = self.padding.get().px();
        let delta = if rtl() {
            rect.right() - viewport.right() + padding
        } else {
            rect.x - viewport.x - padding
        };
        let (offset, limit) = self.boundaries();
        Some((offset + delta).clamp(0.0, limit))
    }

    fn settle_snap(self) {
        let (offset, limit) = self.boundaries();
        if offset <= 1.0 || offset >= limit - 1.0 {
            return;
        }
        let target = (0..self.slides.with(|slides| slides.len()))
            .filter_map(|index| self.slide_offset(index))
            .chain([0.0, limit])
            .min_by(|a, b| (a - offset).abs().total_cmp(&(b - offset).abs()));
        if let Some(target) = target.filter(|target| (target - offset).abs() > 1.0) {
            self.go_to(target);
        }
    }

    fn step(self, next: bool) {
        if !self.can_scroll(next) {
            return;
        }
        let Some(viewport) = self.viewport.rect().get() else {
            return;
        };
        let options = self.options.get();
        let (offset, limit) = self.boundaries();
        let forward = next != rtl();
        if options.looping
            && (if forward {
                offset >= limit - 1.0
            } else {
                offset <= 1.0
            })
        {
            self.go_to(if forward { 0.0 } else { limit });
            return;
        }
        let slides = self.slides.get();
        let first = slides
            .first()
            .and_then(|slide| slide.rect().get())
            .map_or(0.0, |rect| rect.width);
        let distance = (viewport.width - first * 0.5).max(first);
        let target = (offset + if forward { distance } else { -distance }).clamp(0.0, limit);
        let target = if options.snap {
            (0..slides.len())
                .filter_map(|index| self.slide_offset(index))
                .filter(|position| {
                    if forward {
                        *position > offset + 1.0
                    } else {
                        *position < offset - 1.0
                    }
                })
                .min_by(|a, b| (a - target).abs().total_cmp(&(b - target).abs()))
                .unwrap_or(target)
        } else {
            target
        };
        self.go_to(target);
    }
}

pub struct Carousel {
    root: Node,
    label: kivact::Value<String>,
    slides: Vec<View>,
    gap: Space,
    padding: Space,
    buttons: bool,
    edge_fade: bool,
    options: Options,
    handle: Option<CarouselHandle>,
}

pub fn carousel<V: IntoView>(
    label: impl IntoText,
    slides: impl IntoIterator<Item = V>,
) -> Carousel {
    let label = label.into_text_value();
    Carousel {
        root: node().role(Role::Region).name(label),
        label,
        slides: slides.into_iter().map(IntoView::into_view).collect(),
        gap: Space::S1,
        padding: Space::S0,
        buttons: true,
        edge_fade: true,
        options: Options::default(),
        handle: None,
    }
}

impl Carousel {
    pub fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }
    pub fn padding(mut self, padding: Space) -> Self {
        self.padding = padding;
        self
    }
    pub fn buttons(mut self, buttons: bool) -> Self {
        self.buttons = buttons;
        self
    }
    pub fn edge_fade(mut self, fade: bool) -> Self {
        self.edge_fade = fade;
        self
    }
    pub fn looping(mut self, looping: bool) -> Self {
        self.options.looping = looping;
        self
    }
    pub fn snap(mut self, snap: bool) -> Self {
        self.options.snap = snap;
        self
    }
    pub fn handle(mut self, handle: CarouselHandle) -> Self {
        self.handle = Some(handle);
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
            label,
            slides,
            gap,
            padding,
            buttons,
            edge_fade,
            options,
            handle,
        } = self;
        provide((), move || {
            let handle = handle.unwrap_or_else(use_carousel);
            let count = slides.len();
            let refs: Rc<[NodeRef]> = (0..count)
                .map(|_| {
                    let reference = node_ref();
                    let _ = reference.rect();
                    reference
                })
                .collect::<Vec<_>>()
                .into();
            handle.slides.set(refs.clone());
            handle.options.set(options);
            handle.padding.set(padding);
            let slides = slides
                .into_iter()
                .enumerate()
                .map(|(index, slide)| {
                    let name = kivact::i18n::message_with(
                        "soft.carousel.slide",
                        "Slide {current} of {total}",
                        move || {
                            [
                                ("current", (index + 1).to_string()),
                                ("total", count.to_string()),
                            ]
                        },
                    );
                    node()
                        .node_ref(refs[index])
                        .role(Role::Group)
                        .name(name)
                        .style(style().shrink(0))
                        .child(slide)
                })
                .collect::<Vec<_>>();
            let content = row(slides).node_ref(handle.content).style(move || {
                style()
                    .width(Length::MaxContent)
                    .min_width(pct(100))
                    .shrink(0)
                    .padding_x(padding.px())
                    .direction(if rtl() {
                        Direction::RowReverse
                    } else {
                        Direction::Row
                    })
                    .align_items(Align::Center)
                    .gap(gap.px())
            });
            let overflow = use_scroll_overflow(handle.viewport, handle.content);
            if options.snap {
                let offset = handle.viewport.scroll_offset();
                let last = Cell::new(0.0);
                effect_on(
                    move || offset.get().x,
                    move |offset| {
                        if last.replace(*offset) != *offset {
                            let task = spawn(async move {
                                sleep(SNAP_IDLE).await;
                                handle.settle_snap();
                            });
                            on_cleanup(move || task.cancel());
                        }
                    },
                );
            }
            let viewport = crate::scrollable_area::scrollable_area(label, content)
                .axis(ScrollAxis::Inline)
                .node_ref(handle.viewport)
                .on_wheel(move |event: WheelEvent| {
                    if event.modifiers.shift
                        && event.delta.y != 0.0
                        && event.delta.x == 0.0
                        && overflow.any.get()
                    {
                        handle.viewport.scroll_by(
                            Point {
                                x: event.delta.y,
                                y: 0.0,
                            },
                            ScrollBehavior::Instant,
                        );
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                })
                .style(move || {
                    let base = style().scrollbar_width(ScrollbarWidth::None).min_width(0);
                    if !edge_fade || !overflow.any.get() {
                        return base;
                    }
                    let fade_start = options.looping || overflow.start.get();
                    let fade_end = options.looping || overflow.end.get();
                    let mut mask = linear_gradient(if rtl() { 270 } else { 90 });
                    let opaque = Color::BLACK;
                    let middle = rgba(0, 0.3);
                    if fade_start {
                        mask = mask
                            .stop(Color::TRANSPARENT, 0)
                            .stop(middle, 2)
                            .stop(opaque, 4);
                    } else {
                        mask = mask.stop(opaque, 0);
                    }
                    if fade_end {
                        mask = mask
                            .stop(opaque, pct(100) - 4)
                            .stop(middle, pct(100) - 2)
                            .stop(Color::TRANSPARENT, pct(100));
                    } else {
                        mask = mask.stop(opaque, pct(100));
                    }
                    base.mask(mask)
                });
            let controls = buttons.then(|| (pill(handle, false), pill(handle, true)));
            root.base_style(
                style()
                    .position(Position::Relative)
                    .direction(Direction::Column)
                    .min_width(0)
                    .max_width(pct(100)),
            )
            .child((viewport, controls))
        })
    }
}

fn pill(handle: CarouselHandle, next: bool) -> impl IntoView {
    let physical_right = next != rtl();
    let label = if physical_right {
        kivact::i18n::message("soft.carousel.scroll_right", "Scroll right")
    } else {
        kivact::i18n::message("soft.carousel.scroll_left", "Scroll left")
    };
    let reference = node_ref();
    let focused = reference.focused();
    effect_on(
        move || handle.can_scroll(next),
        move |available| {
            if !available && focused.get() {
                handle.viewport.focus();
            }
        },
    );
    let glyph = icon(if physical_right {
        "chevron-right"
    } else {
        "chevron-left"
    })
    .style(|| {
        let size = tokens().sizes.icon_xsm;
        style().size(size, size)
    });
    button(label)
        .variant(ButtonVariant::Ghost)
        .size(ControlSize::Small)
        .icon(glyph)
        .hide_label(true)
        .node_ref(reference)
        .disabled(move || !handle.can_scroll(next))
        .on_press(move || handle.step(next))
        .style(move || {
            let t = tokens();
            let visible = handle.can_scroll(next);
            let size = t.sizes.control_sm;
            let base = style()
                .position(Position::Absolute)
                .top(pct(50) - size / 2.0)
                .z_index(1)
                .size(size, size)
                .radius(t.radius.full)
                .background(t.colors.bg_popover)
                .opacity(if visible { 1.0 } else { 0.0 })
                .transition(
                    TransitionProp::Paint,
                    Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
                );
            let mut base = if physical_right {
                base.right(-size / 2.0)
            } else {
                base.left(-size / 2.0)
            };
            for shadow in &t.shadows.med {
                base = base.shadow(*shadow);
            }
            base
        })
}
