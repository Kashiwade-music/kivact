//! Role: Render Neutral progress steps and a width-sensitive summary.
use crate::side_nav::logical_row;
use crate::stack::rtl;
use crate::tokens;
use kivact::control::{Node, NodeBuilder, column, icon, node, row, text};
use kivact::i18n::message;
use kivact::motion::Transition;
use kivact::primitive::NodeExt;
use kivact::style::{Align, Color, Direction, Justify, TransitionProp, pct, style};
use kivact::ui::{ButtonVariant, ControlSize, button};
use kivact::view::{
    HandlerResult, IntoText, IntoValue, IntoView, Orientation, Role, View, callback, node_ref,
    provide,
};
use kivact::{Callback, Value, memo};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepDensity {
    Compact,
    #[default]
    Balanced,
    Spacious,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepIndicatorPosition {
    #[default]
    Separated,
    OnTrack,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepCollapsedVariant {
    #[default]
    WithLabelAndControls,
    WithLabel,
    HiddenLabel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepStatus {
    #[default]
    Default,
    Accent,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Default)]
pub enum StepIndicator {
    #[default]
    Auto,
    Number,
    None,
    Custom(Rc<dyn Fn() -> View>),
}

impl StepIndicator {
    pub fn custom<V: IntoView>(glyph: impl Fn() -> V + 'static) -> Self {
        Self::Custom(Rc::new(move || glyph().into_view()))
    }
}

pub struct Step {
    root: Node,
    index: usize,
    label: Value<String>,
    description: Option<Value<String>>,
    content: Option<View>,
    end: Option<View>,
    disabled: Value<bool>,
    optional: bool,
    status: StepStatus,
    indicator: StepIndicator,
    density: Option<StepDensity>,
}

pub fn step(index: usize, label: impl IntoText) -> Step {
    let label = label.into_text_value();
    Step {
        root: node().role(Role::ListItem).name(label),
        index,
        label,
        description: None,
        content: None,
        end: None,
        disabled: false.into_value(),
        optional: false,
        status: StepStatus::Default,
        indicator: StepIndicator::Auto,
        density: None,
    }
}

impl Step {
    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(description.into_text_value());
        self
    }
    pub fn content<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.content = Some(provide((), content));
        self
    }
    pub fn end_content<V: IntoView>(mut self, end: impl FnOnce() -> V + 'static) -> Self {
        self.end = Some(provide((), end));
        self
    }
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }
    pub fn optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }
    pub fn status(mut self, status: StepStatus) -> Self {
        self.status = status;
        self
    }
    pub fn indicator(mut self, indicator: StepIndicator) -> Self {
        self.indicator = indicator;
        self
    }
    pub fn density(mut self, density: StepDensity) -> Self {
        self.density = Some(density);
        self
    }
}

impl NodeBuilder for Step {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

pub struct Stepper {
    root: Node,
    active: Value<usize>,
    steps: Vec<Step>,
    orientation: Orientation,
    action: Option<Callback<usize>>,
    density: StepDensity,
    position: StepIndicatorPosition,
    min_width: f64,
    collapsed: StepCollapsedVariant,
    connector_gap: f64,
}

pub fn stepper(active: impl IntoValue<usize>, steps: impl IntoIterator<Item = Step>) -> Stepper {
    Stepper {
        root: node().name(message("soft.stepper.label", "Progress")),
        active: active.into_value(),
        steps: steps.into_iter().collect(),
        orientation: Orientation::Horizontal,
        action: None,
        density: StepDensity::Balanced,
        position: StepIndicatorPosition::Separated,
        min_width: 112.0,
        collapsed: StepCollapsedVariant::WithLabelAndControls,
        connector_gap: 0.0,
    }
}

impl Stepper {
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn on_step_press<R: HandlerResult>(
        mut self,
        action: impl Fn(usize) -> R + 'static,
    ) -> Self {
        self.action = Some(callback(action));
        self
    }
    pub fn density(mut self, density: StepDensity) -> Self {
        self.density = density;
        self
    }
    pub fn indicator_position(mut self, position: StepIndicatorPosition) -> Self {
        self.position = position;
        self
    }
    pub fn min_step_width(mut self, width: f64) -> Self {
        self.min_width = width.max(0.0);
        self
    }
    pub fn collapsed_variant(mut self, variant: StepCollapsedVariant) -> Self {
        self.collapsed = variant;
        self
    }
    pub fn connector_gap(mut self, gap: f64) -> Self {
        self.connector_gap = gap.clamp(0.0, 8.0);
        self
    }
}

impl NodeBuilder for Stepper {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

#[derive(Clone, Copy)]
struct StepperCx {
    active: Value<usize>,
    compact: Value<bool>,
    vertical: bool,
    action: Option<Callback<usize>>,
    density: StepDensity,
    position: StepIndicatorPosition,
    connector_gap: f64,
}

impl IntoView for Stepper {
    fn into_view(self) -> View {
        let Self {
            root,
            active,
            steps,
            orientation,
            action,
            density,
            position,
            min_width,
            collapsed,
            connector_gap,
        } = self;
        provide((), move || {
            let root_ref = node_ref();
            let rect = root_ref.rect();
            let count = steps.len();
            let vertical = orientation == Orientation::Vertical;
            let compact = memo(move || {
                !vertical
                    && count > 0
                    && rect
                        .get()
                        .is_some_and(|r| r.width / (count as f64) < min_width)
            });
            let cx = StepperCx {
                active,
                compact,
                vertical,
                action,
                density,
                position,
                connector_gap,
            };
            let summaries: Rc<[_]> = steps
                .iter()
                .map(Summary::from_step)
                .collect::<Vec<_>>()
                .into();
            let list = node()
                .role(Role::List)
                .style(move || {
                    let t = tokens();
                    style()
                        .direction(if vertical {
                            Direction::Column
                        } else {
                            logical_row()
                        })
                        .gap(
                            if !vertical
                                && position == StepIndicatorPosition::OnTrack
                                && !compact.get()
                            {
                                0.0
                            } else {
                                t.spacing.s1
                            },
                        )
                })
                .child(
                    steps
                        .into_iter()
                        .enumerate()
                        .map(|(i, step)| step.build(cx, i == 0, i + 1 == count))
                        .collect::<Vec<_>>(),
                );
            let summary = (collapsed != StepCollapsedVariant::HiddenLabel).then(|| {
                let step_labels = summaries.clone();
                let summary = kivact::view::dynamic(active, move |index| {
                    step_labels
                        .iter()
                        .find(|step| step.index == index)
                        .map(|step| {
                            column((
                                row((
                                    step_indicator(step.clone(), cx)(),
                                    text(step.label).style(move || step_text(true, false, false)),
                                ))
                                .style(|| {
                                    let t = tokens();
                                    style()
                                        .direction(logical_row())
                                        .align_items(Align::Center)
                                        .justify_content(Justify::Center)
                                        .gap(t.spacing.s2)
                                }),
                                step.description
                                    .map(|description| text(description).style(description_style)),
                            ))
                            .style(|| {
                                style()
                                    .align_items(Align::Center)
                                    .gap(tokens().spacing.s0_5)
                            })
                        })
                });
                let controls =
                    action.is_some() && collapsed == StepCollapsedVariant::WithLabelAndControls;
                row((
                    controls.then(|| summary_button(cx, summaries.clone(), false)),
                    summary,
                    controls.then(|| summary_button(cx, summaries.clone(), true)),
                ))
                .hidden(move || !compact.get())
                .style(move || {
                    style()
                        .direction(logical_row())
                        .align_items(Align::Center)
                        .gap(tokens().spacing.s2)
                        .justify_content(if controls {
                            Justify::SpaceBetween
                        } else {
                            Justify::Center
                        })
                })
            });
            root.node_ref(root_ref)
                .base_style(|| {
                    style()
                        .direction(Direction::Column)
                        .gap(tokens().spacing.s2)
                })
                .child((list, summary))
        })
    }
}

#[derive(Clone)]
struct Summary {
    index: usize,
    label: Value<String>,
    description: Option<Value<String>>,
    disabled: Value<bool>,
    indicator: StepIndicator,
    status: StepStatus,
}

impl Summary {
    fn from_step(step: &Step) -> Self {
        Self {
            index: step.index,
            label: step.label,
            description: step.description,
            disabled: step.disabled,
            indicator: step.indicator.clone(),
            status: step.status,
        }
    }
}

impl IntoView for Step {
    fn into_view(self) -> View {
        provide((), move || {
            let cx = StepperCx {
                active: self.index.into_value(),
                compact: false.into_value(),
                vertical: false,
                action: None,
                density: StepDensity::Balanced,
                position: StepIndicatorPosition::Separated,
                connector_gap: 0.0,
            };
            self.build(cx, true, true)
        })
    }
}

impl Step {
    fn build(self, cx: StepperCx, first: bool, last: bool) -> Node {
        let summary = Summary::from_step(&self);
        let Self {
            root,
            index,
            label,
            description,
            content,
            end,
            disabled,
            optional,
            density,
            ..
        } = self;
        let density = density.unwrap_or(cx.density);
        let on_track = cx.position == StepIndicatorPosition::OnTrack;
        let has_indicator = !matches!(summary.indicator, StepIndicator::None);
        let bar = node()
            .hidden(move || on_track && !cx.compact.get())
            .style(move || {
                let t = tokens();
                let base = style()
                    .radius(t.radius.full)
                    .shrink(0)
                    .background(if index <= cx.active.get() {
                        t.colors.accent
                    } else {
                        t.colors.border
                    })
                    .transition(
                        TransitionProp::Paint,
                        Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
                    );
                if cx.vertical {
                    base.width(t.spacing.s1).align_self(Align::Stretch)
                } else {
                    base.width(pct(100)).height(t.spacing.s1)
                }
            });
        let indicator = step_indicator(summary, cx);
        let connector = |leading: bool, absent: bool| {
            node().style(move || {
                let t = tokens();
                let filled = if leading {
                    index <= cx.active.get()
                } else {
                    index < cx.active.get()
                };
                let base = style()
                    .grow(1)
                    .opacity(if absent { 0.0 } else { 1.0 })
                    .background(if filled {
                        t.colors.accent
                    } else {
                        t.colors.border
                    });
                if cx.vertical {
                    base.width(t.spacing.s1).min_height(t.spacing.s2)
                } else {
                    base.height(t.spacing.s1).min_width(t.spacing.s2)
                }
            })
        };
        let track = on_track.then(|| {
            let gap = cx.connector_gap;
            node()
                .style(move || {
                    style()
                        .direction(if cx.vertical {
                            Direction::Column
                        } else {
                            logical_row()
                        })
                        .align_items(Align::Center)
                        .gap(gap)
                        .shrink(0)
                })
                .child((connector(true, first), indicator(), connector(false, last)))
        });
        let label_node = text(label).style(move || {
            step_text(
                cx.active.get() == index,
                cx.active.get() < index,
                disabled.get(),
            )
        });
        let optional = optional.then(|| {
            text(message("soft.stepper.optional", "• Optional")).style(|| {
                style()
                    .color(tokens().colors.text_secondary)
                    .text_wrap(false)
            })
        });
        let leading = if on_track { None } else { Some(indicator()) };
        let label_row = row((leading, label_node, optional, end)).style(move || {
            let t = tokens();
            style()
                .direction(logical_row())
                .align_items(Align::Center)
                .justify_content(if on_track && !cx.vertical {
                    Justify::Center
                } else {
                    Justify::Start
                })
                .gap(if on_track { t.spacing.s1 } else { t.spacing.s2 })
        });
        let description = description.map(|description| {
            text(description).style(move || {
                let t = tokens();
                let base = description_style();
                if has_indicator && !on_track {
                    if rtl() {
                        base.padding_right(t.spacing.s6)
                    } else {
                        base.padding_left(t.spacing.s6)
                    }
                } else {
                    base
                }
            })
        });
        let labels = column((label_row, description)).style(move || {
            let t = tokens();
            let base = style()
                .padding_y(density_padding(density))
                .padding_x(if density == StepDensity::Spacious {
                    t.spacing.s3
                } else {
                    t.spacing.s2
                })
                .radius(t.radius.element)
                .min_width(0);
            if cx.action.is_some() && !disabled.get() {
                base.hover(style().background(t.colors.overlay_hover))
                    .pressed(style().background(t.colors.overlay_pressed))
                    .focus_visible(crate::recipe_style::ring(&t, t.colors.accent))
            } else {
                base
            }
        });
        let labels = if let Some(action) = cx.action {
            labels
                .role(Role::Button)
                .name(label)
                .focusable(move || !disabled.get())
                .disabled(disabled)
                .selected(move || cx.active.get() == index)
                .on_press(move || {
                    if !disabled.get() {
                        action.call(index);
                    }
                })
        } else {
            labels
        };
        let content = content.map(|content| {
            node()
                .style(move || {
                    let t = tokens();
                    let padding = if density == StepDensity::Spacious {
                        t.spacing.s3
                    } else {
                        t.spacing.s2
                    };
                    let start = padding
                        + if has_indicator && !on_track {
                            t.spacing.s6
                        } else {
                            0.0
                        };
                    let base = style().padding_top(t.spacing.s2);
                    if rtl() {
                        base.padding_right(start).padding_left(padding)
                    } else {
                        base.padding_left(start).padding_right(padding)
                    }
                })
                .child(content)
        });
        let body = column((labels, content))
            .hidden(cx.compact)
            .style(style().grow(1).min_width(0));
        let track = track.map(|track| track.hidden(cx.compact));
        root.base_style(move || {
            let t = tokens();
            let base = style().min_width(0).gap(t.spacing.s0_5);
            if cx.vertical {
                base.direction(logical_row()).align_items(Align::Stretch)
            } else {
                base.direction(Direction::Column).grow(1).basis(0)
            }
        })
        .selected(move || cx.active.get() == index)
        .current_item(move || (cx.active.get() == index).then_some(kivact::view::CurrentItem::Step))
        .child((bar, track, body))
    }
}

fn step_indicator(step: Summary, cx: StepperCx) -> Rc<dyn Fn() -> View> {
    Rc::new(move || {
        let Summary {
            index,
            disabled,
            indicator,
            status,
            ..
        } = step.clone();
        if matches!(indicator, StepIndicator::None) {
            return View::empty();
        }
        let number = matches!(indicator, StepIndicator::Number);
        let custom = if let StepIndicator::Custom(glyph) = indicator {
            Some(glyph)
        } else {
            None
        };
        let content = kivact::view::dynamic(cx.active, move |active| {
            if let Some(custom) = &custom {
                return custom();
            }
            let (background, foreground) = indicator_colors(status);
            let reached = index <= active;
            if !number && index == active {
                return node()
                    .style(move || {
                        let t = tokens();
                        style()
                            .size(t.spacing.s4, t.spacing.s4)
                            .radius(t.radius.full)
                            .border(t.spacing.s0_5, background)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                    })
                    .child(
                        node().style(move || style().size(8, 8).radius(4).background(background)),
                    )
                    .into_view();
            }
            if !number
                && matches!(
                    status,
                    StepStatus::Success | StepStatus::Warning | StepStatus::Error
                )
            {
                let (glyph, color) = match status {
                    StepStatus::Success => ("circle-check", tokens().colors.text_success),
                    StepStatus::Warning => ("triangle-alert", tokens().colors.text_warning),
                    _ => ("circle-x", tokens().colors.text_error),
                };
                return icon(glyph)
                    .style(move || style().size(16, 16).color(color))
                    .into_view();
            }
            let badge = move || {
                let t = tokens();
                let base = style()
                    .size(t.spacing.s4, t.spacing.s4)
                    .radius(t.radius.full)
                    .justify_content(Justify::Center)
                    .align_items(Align::Center)
                    .font_size(t.typography.size_xs)
                    .weight(t.typography.weight_semibold);
                if reached {
                    base.background(background).color(foreground)
                } else {
                    base.background(t.colors.bg_muted).color(if disabled.get() {
                        t.colors.text_disabled
                    } else {
                        t.colors.text_secondary
                    })
                }
            };
            let glyph = if !number && reached {
                icon("check").style(style().size(12, 12)).into_view()
            } else {
                text(index.saturating_add(1).to_string()).into_view()
            };
            node().style(badge).child(glyph).into_view()
        });
        node()
            .style(move || {
                let t = tokens();
                style()
                    .size(t.spacing.s4, t.spacing.s4)
                    .shrink(0)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .opacity(if disabled.get() {
                        t.sizes.disabled_opacity
                    } else {
                        1.0
                    })
            })
            .child(content)
            .into_view()
    })
}

fn indicator_colors(status: StepStatus) -> (Color, Color) {
    let c = tokens().colors;
    match status {
        StepStatus::Default => (c.accent, c.bg_surface),
        StepStatus::Accent => (c.accent, c.on_accent),
        StepStatus::Success => (c.success, c.on_success),
        StepStatus::Warning => (c.warning, c.on_warning),
        StepStatus::Error => (c.error, c.on_error),
    }
}

fn density_padding(density: StepDensity) -> f64 {
    let t = tokens();
    match density {
        StepDensity::Compact => t.spacing.s1,
        StepDensity::Balanced => t.spacing.s2,
        StepDensity::Spacious => t.spacing.s3,
    }
}

fn step_text(active: bool, future: bool, disabled: bool) -> kivact::style::Style {
    let t = tokens();
    let role = t.typography.body;
    style()
        .font_size(role.size_px)
        .line_height(role.line_height_px / role.size_px)
        .color(if disabled {
            t.colors.text_disabled
        } else if future {
            t.colors.text_secondary
        } else {
            t.colors.text_primary
        })
        .weight(if active {
            t.typography.weight_semibold
        } else {
            role.weight
        })
        .single_line()
}

fn description_style() -> kivact::style::Style {
    let t = tokens();
    style()
        .font_size(t.typography.supporting.size_px)
        .line_height(t.spacing.s4 / t.typography.supporting.size_px)
        .color(t.colors.text_secondary)
}

fn summary_button(cx: StepperCx, steps: Rc<[Summary]>, next: bool) -> impl IntoView {
    let label = if next {
        message("soft.stepper.next", "Next step")
    } else {
        message("soft.stepper.previous", "Previous step")
    };
    let target = memo(move || {
        let active = cx.active.get();
        let mut candidates = steps.iter().filter(|step| !step.disabled.get());
        if next {
            candidates.find(|step| step.index > active)
        } else {
            candidates.rfind(|step| step.index < active)
        }
        .map(|step| step.index)
    });
    let glyph = if next != rtl() {
        "chevron-right"
    } else {
        "chevron-left"
    };
    button(label)
        .variant(ButtonVariant::Ghost)
        .size(ControlSize::Medium)
        .icon(icon(glyph))
        .hide_label(true)
        .disabled(move || target.get().is_none())
        .on_press(move || {
            if let (Some(action), Some(index)) = (cx.action, target.get()) {
                action.call(index);
            }
        })
}
