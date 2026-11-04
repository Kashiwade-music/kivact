//! Role: Resize and optionally collapse a Neutral layout panel.
use crate::stack::rtl;
use crate::tokens;
use kivact::interaction::ScalarRange;
use kivact::interaction::use_splitter;
use kivact::motion::Transition;
use kivact::state::IntoBinding;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::node;

use kivact::state::Binding;
use kivact::state::Signal;
use kivact::state::binding;
use kivact::state::provide;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Color;
use kivact::style::Cursor;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::Handled;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::KeyEvent;
use kivact::view::NodeRef;
use kivact::view::Orientation;
use kivact::view::View;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResizeSize {
    Px(f64),
    Percent {
        value: f64,
        floor: Option<f64>,
        ceil: Option<f64>,
    },
}

impl ResizeSize {
    fn resolve(self, basis: f64) -> f64 {
        match self {
            Self::Px(px) => px,
            Self::Percent { value, floor, ceil } => {
                let min = floor.unwrap_or(0.0);
                let max = ceil.unwrap_or(f64::MAX).max(min);
                (basis * value / 100.0).clamp(min, max)
            }
        }
    }
}

pub struct ResizableConfig {
    pub default_size: ResizeSize,
    pub min_size: ResizeSize,
    pub max_size: ResizeSize,
    pub container: Option<NodeRef>,
    pub axis: Orientation,
    pub collapsible: bool,
    pub collapse_threshold: f64,
    pub snaps: Vec<f64>,
    pub default_collapsed: bool,
    pub collapsed: Option<Binding<bool>>,
}

impl Default for ResizableConfig {
    fn default() -> Self {
        Self {
            default_size: ResizeSize::Px(250.0),
            min_size: ResizeSize::Px(50.0),
            max_size: ResizeSize::Px(f64::MAX),
            container: None,
            axis: Orientation::Horizontal,
            collapsible: false,
            collapse_threshold: 40.0,
            snaps: Vec::new(),
            default_collapsed: false,
            collapsed: None,
        }
    }
}

#[derive(Clone)]
pub struct ResizableState {
    expanded_size: Signal<f64>,
    collapsed: Binding<bool>,
    config: Rc<ResizableConfig>,
}

pub fn use_resizable(config: ResizableConfig) -> ResizableState {
    let collapsed = config
        .collapsed
        .unwrap_or_else(|| signal(config.default_collapsed).into_binding());
    let basis = basis(&config);
    let expanded_size = signal(config.default_size.resolve(basis).max(0.0));
    ResizableState {
        expanded_size,
        collapsed,
        config: Rc::new(config),
    }
}

fn basis(config: &ResizableConfig) -> f64 {
    let vertical = config.axis == Orientation::Vertical;
    if let Some(node) = config.container {
        return node.rect().get().map_or(
            1200.0,
            |rect| if vertical { rect.height } else { rect.width },
        );
    }
    let size = kivact::platform::window_size().get();
    let axis = if vertical { size.height } else { size.width };
    if axis > 0.0 { axis } else { 1200.0 }
}

impl ResizableState {
    fn limits(&self) -> (f64, f64) {
        let basis = basis(&self.config);
        let min = self.config.min_size.resolve(basis).max(0.0);
        let max = self.config.max_size.resolve(basis).max(min);
        (min, max)
    }

    pub fn size(&self) -> f64 {
        if self.is_collapsed() {
            0.0
        } else {
            let (min, max) = self.limits();
            self.expanded_size.get().clamp(min, max)
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.config.collapsible && self.collapsed.get()
    }

    pub fn collapse(&self) {
        if self.config.collapsible {
            self.collapsed.set(true);
        }
    }

    pub fn expand(&self) {
        self.collapsed.set(false);
    }

    pub fn resize(&self, requested: impl Into<f64>) {
        let requested = requested.into();
        if !requested.is_finite() {
            return;
        }
        if self.config.collapsible && requested < self.config.collapse_threshold {
            self.collapse();
            return;
        }
        let (min, max) = self.limits();
        let size = self
            .config
            .snaps
            .iter()
            .copied()
            .filter(|point| *point >= min && *point <= max)
            .min_by(|a, b| (a - requested).abs().total_cmp(&(b - requested).abs()))
            .unwrap_or(requested);
        self.expanded_size.set(size.clamp(min, max));
        self.expand();
    }

    fn keyboard_resize(&self, forward: bool, shift: bool) {
        let (min, max) = self.limits();
        let size = self.size();
        let step = if shift { 50.0 } else { 10.0 };
        if self.is_collapsed() {
            if forward {
                self.resize((min + step).max(self.config.collapse_threshold));
            }
            return;
        }
        let mut points = self
            .config
            .snaps
            .iter()
            .copied()
            .filter(|point| {
                *point >= min
                    && *point <= max
                    && if forward {
                        *point > size
                    } else {
                        *point < size
                    }
            })
            .collect::<Vec<_>>();
        points.sort_by(f64::total_cmp);
        if !forward {
            points.reverse();
        }
        let target = if self.config.snaps.is_empty() {
            (if forward { size + step } else { size - step }).clamp(min, max)
        } else {
            points
                .get(usize::from(shift))
                .or_else(|| points.last())
                .copied()
                .unwrap_or(size)
        };
        self.expanded_size.set(target);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PillPlacement {
    Start,
    End,
    Center,
    #[default]
    Auto,
}

pub struct ResizeHandle {
    root: Node,
    state: ResizableState,
    axis: Value<Orientation>,
    reversed: Value<bool>,
    divider: Value<bool>,
    always_visible: Value<bool>,
    overlay: Value<bool>,
    placement: Value<PillPlacement>,
    grip: Option<View>,
}

pub fn resize_handle(state: ResizableState) -> ResizeHandle {
    let axis = state.config.axis;
    ResizeHandle {
        root: node().name(kivact::i18n::message(
            "soft.resize_handle.label",
            "Resize handle",
        )),
        state,
        axis: (axis).into_value(),
        reversed: (false).into_value(),
        divider: (false).into_value(),
        always_visible: (true).into_value(),
        overlay: (false).into_value(),
        placement: (PillPlacement::Auto).into_value(),
        grip: None,
    }
}

impl ResizeHandle {
    pub fn axis(mut self, axis: impl IntoValue<Orientation>) -> Self {
        self.axis = axis.into_value();
        self
    }
    pub fn reversed(mut self, reversed: impl IntoValue<bool>) -> Self {
        self.reversed = reversed.into_value();
        self
    }
    pub fn divider(mut self, divider: impl IntoValue<bool>) -> Self {
        self.divider = divider.into_value();
        self
    }
    pub fn always_visible(mut self, always: impl IntoValue<bool>) -> Self {
        self.always_visible = always.into_value();
        self
    }
    pub fn overlay(mut self, overlay: impl IntoValue<bool>) -> Self {
        self.overlay = overlay.into_value();
        self
    }
    pub fn pill_placement(mut self, placement: impl IntoValue<PillPlacement>) -> Self {
        self.placement = placement.into_value();
        self
    }
    pub fn grip(mut self, grip: impl IntoView) -> Self {
        self.grip = Some(provide((), move || grip.into_view()));
        self
    }
}

impl NodeBuilder for ResizeHandle {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for ResizeHandle {
    fn into_view(self) -> View {
        let Self {
            mut root,
            state,
            axis,
            reversed,
            divider,
            always_visible,
            overlay,
            placement,
            grip,
        } = self;
        provide((), move || {
            let vertical = axis.get() == Orientation::Vertical;
            let axis = if vertical {
                Orientation::Vertical
            } else {
                Orientation::Horizontal
            };
            let mirror = reversed.get() != (!vertical && rtl());
            let scalar = {
                let get = state.clone();
                let set = state.clone();
                binding(move || get.size(), move |value| set.resize(value))
            };
            let range = {
                let state = state.clone();
                move || {
                    let (min, max) = state.limits();
                    let lower = if state.config.collapsible { 0.0 } else { min };
                    ScalarRange::new(lower, max).expect("valid resize range")
                }
            };
            let mut behavior = use_splitter(scalar, range, axis)
                .key_step(10.0)
                .expect("positive key step");
            if reversed.get() {
                behavior = behavior.pointer_scale(-1.0);
            }
            let disabled = root.accessible_disabled();
            if let Some(disabled) = disabled {
                behavior = behavior.disabled(move || disabled.get());
            }
            let handle = behavior.handle();
            let visual = handle.state();
            let key_state = state.clone();
            let root = root
                .with_node(move |node| node.part(handle))
                .on_key(move |event: KeyEvent| {
                    if disabled.as_ref().is_some_and(|disabled| disabled.get())
                        || event.modifiers().ctrl
                        || event.modifiers().alt
                        || event.modifiers().meta
                    {
                        return Handled::No;
                    }
                    let forward = match (vertical, event.key()) {
                        (false, Key::ArrowLeft) | (true, Key::ArrowUp) => false,
                        (false, Key::ArrowRight) | (true, Key::ArrowDown) => true,
                        (_, Key::Home) => {
                            key_state.resize(key_state.limits().0);
                            return Handled::Yes;
                        }
                        (_, Key::End) if key_state.limits().1 < f64::MAX => {
                            key_state.resize(key_state.limits().1);
                            return Handled::Yes;
                        }
                        (_, Key::End) => return Handled::Yes,
                        (_, Key::Enter) if key_state.config.collapsible => {
                            if !event.repeat() {
                                if key_state.is_collapsed() {
                                    key_state.expand();
                                } else {
                                    key_state.collapse();
                                }
                            }
                            return Handled::Yes;
                        }
                        _ => return Handled::No,
                    };
                    key_state.keyboard_resize(forward != mirror, event.modifiers().shift);
                    Handled::Yes
                })
                .with_node(move |node| {
                    node.base_style(move || {
                        let t = tokens();
                        let base = style()
                            .direction(Direction::Column)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                            .align_self(Align::Stretch)
                            .shrink(0)
                            .cursor(if vertical {
                                Cursor::NsResize
                            } else {
                                Cursor::EwResize
                            })
                            .focus_visible(crate::recipe_style::ring(&t, t.colors.accent));
                        let hit = t.spacing.s4;
                        if overlay.get() {
                            let base = base.position(Position::Absolute).z_index(2);
                            if vertical {
                                base.left(0).right(0).bottom(0).height(hit)
                            } else if rtl() {
                                base.top(0).bottom(0).left(0).width(hit)
                            } else {
                                base.top(0).bottom(0).right(0).width(hit)
                            }
                        } else {
                            let line = if divider.get() {
                                t.sizes.border_width
                            } else {
                                0.0
                            };
                            let extra = (hit - line) / -2.0;
                            if vertical {
                                base.height(hit).margin_y(extra)
                            } else {
                                base.width(hit).margin_x(extra)
                            }
                        }
                    })
                });
            let rule = node().style(move || {
                let t = tokens();
                let dragging = visual.get().dragging;
                let color = if dragging {
                    t.colors.border_emphasized
                } else {
                    t.colors.border
                };
                let line = style()
                    .position(Position::Absolute)
                    .background(if divider.get() {
                        color
                    } else {
                        Color::TRANSPARENT
                    });
                if vertical {
                    line.width(pct(100)).height(t.sizes.border_width)
                } else {
                    line.height(pct(100)).width(t.sizes.border_width)
                }
            });
            let pill = grip.unwrap_or_else(|| {
                node()
                    .style(move || {
                        let t = tokens();
                        let state_now = visual.get();
                        let visible = always_visible.get()
                            || state_now.dragging
                            || state_now.input.hovered
                            || state_now.input.focused;
                        let side = match placement.get() {
                            PillPlacement::Start => -1.0,
                            PillPlacement::End => 1.0,
                            PillPlacement::Center => 0.0,
                            PillPlacement::Auto => {
                                if reversed.get() != state.is_collapsed() {
                                    1.0
                                } else {
                                    -1.0
                                }
                            }
                        };
                        let pill = style()
                            .radius(t.radius.full)
                            .opacity(if visible { 1.0 } else { 0.0 })
                            .background(if state_now.dragging {
                                t.colors.border_emphasized
                            } else {
                                t.colors.border
                            })
                            .transition(
                                TransitionProp::Paint,
                                Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
                            );
                        if vertical {
                            pill.size(t.spacing.s8, 3).translate_y(side * 7.0)
                        } else {
                            pill.size(3, t.spacing.s8)
                                .translate_x(side * if rtl() { -7.0 } else { 7.0 })
                        }
                    })
                    .into_view()
            });
            root.child((rule, pill))
        })
    }
}

kivact::state::constant_value!(PillPlacement);
