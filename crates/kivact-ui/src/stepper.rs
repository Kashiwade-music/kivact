//! Role: Render reactive progress steps with semantic current-step metadata.
use crate::navigation_values::{StepStatus, StepperItemState, StepperState};
use kivact_core::custom::Reactive;
use kivact_core::state::{IntoText, IntoValue, Value, memo};
use kivact_core::style::{Direction, style};
use kivact_core::view::{
    CurrentItem, IntoView, Node, NodeBuilder, Orientation, Role, View, column, for_index, node,
    node_ref, show, text,
};
use std::rc::Rc;
/// A step's optional indicator.
#[derive(Clone, Default, PartialEq)]
pub enum StepIndicator {
    #[default]
    Auto,
    Number,
    None,
    Custom(IndicatorContent),
}
/// A custom indicator factory; equality follows factory identity.
#[derive(Clone)]
pub struct IndicatorContent(Rc<dyn Fn() -> View>);
impl PartialEq for IndicatorContent {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl StepIndicator {
    /// Build custom indicator contents at each mount.
    pub fn custom<V: IntoView>(f: impl Fn() -> V + 'static) -> Self {
        Self::Custom(IndicatorContent(Rc::new(move || f().into_view())))
    }
}
/// Passive step metadata; labels can be localized or reactive.
#[derive(Clone, PartialEq)]
pub struct StepperItem {
    label: Reactive<String>,
    description: Option<Reactive<String>>,
    status: StepStatus,
    disabled: Reactive<bool>,
    optional: Reactive<bool>,
    indicator: StepIndicator,
    content: Option<Content>,
    end_content: Option<Content>,
}
#[derive(Clone)]
struct Content(Rc<dyn Fn() -> View>);
impl PartialEq for Content {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl StepperItem {
    /// Create a step with the given visible label.
    pub fn new(label: impl IntoText) -> Self {
        Self {
            label: Reactive::new(label.into_text()),
            description: None,
            status: StepStatus::Default,
            disabled: Reactive::new(false),
            optional: Reactive::new(false),
            indicator: StepIndicator::Auto,
            content: None,
            end_content: None,
        }
    }
    /// Add supporting text beneath the label.
    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(Reactive::new(description.into_text()));
        self
    }
    /// Set semantic status for recipes and assistive descriptions.
    pub fn status(mut self, status: StepStatus) -> Self {
        self.status = status;
        self
    }
    /// Prevent activation of this step.
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = Reactive::new(disabled);
        self
    }
    /// Mark a step as optional.
    pub fn optional(mut self, optional: impl IntoValue<bool>) -> Self {
        self.optional = Reactive::new(optional);
        self
    }
    /// Choose automatic completion marks, numbers, or no indicator.
    pub fn indicator(mut self, indicator: StepIndicator) -> Self {
        self.indicator = indicator;
        self
    }
    /// Add custom contents beneath the label.
    pub fn content<V: IntoView>(mut self, content: impl Fn() -> V + 'static) -> Self {
        self.content = Some(Content(Rc::new(move || content().into_view())));
        self
    }
    /// Add trailing contents.
    pub fn end_content<V: IntoView>(mut self, content: impl Fn() -> V + 'static) -> Self {
        self.end_content = Some(Content(Rc::new(move || content().into_view())));
        self
    }
    /// Read the visible label.
    pub fn label(&self) -> String {
        self.label.get()
    }
}
/// A progress list whose current step belongs to the application.
pub struct Stepper {
    root: Node,
    items: Value<Vec<StepperItem>>,
    current: Value<usize>,
    orientation: Value<Orientation>,
    activate: Option<Rc<dyn Fn(usize)>>,
    // xtask-allow(state-fields): Independent setup-only override of the passive minimum width token.
    min_step_width: Option<Value<f64>>,
    variants: crate::variants::Variants,
}
/// Show progress through an ordered sequence of steps.
#[track_caller]
pub fn stepper(items: impl IntoValue<Vec<StepperItem>>, current: impl IntoValue<usize>) -> Stepper {
    Stepper {
        root: node()
            .role(Role::List)
            .name(kivact_core::i18n::message("stepper.label", "Progress")),
        items: items.into_value(),
        current: current.into_value(),
        orientation: Orientation::Horizontal.into_value(),
        activate: None,
        min_step_width: None,
        variants: crate::variants::Variants::default(),
    }
}
impl Stepper {
    /// Override the minimum width used to derive the collapsed layout.
    pub fn min_step_width(mut self, width: impl IntoValue<f64>) -> Self {
        self.min_step_width = Some(width.into_value());
        self
    }
    /// Choose horizontal or vertical layout.
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        self.orientation = orientation.into_value();
        self
    }
    /// Activate an enabled step without changing the caller-owned current value.
    pub fn on_step_press(mut self, f: impl Fn(usize) + 'static) -> Self {
        self.activate = Some(Rc::new(f));
        self
    }
    /// Add kit-defined visual metadata.
    pub fn variant(self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for Stepper {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Stepper {
    fn into_view(self) -> View {
        let Self {
            root,
            items,
            current,
            orientation,
            activate,
            min_step_width,
            variants,
        } = self;
        let reference = node_ref();
        let layout = responsive_layout(items, current, orientation, min_step_width, reference);
        let previous =
            collapsed_navigation(activate.clone(), current, layout, variants.clone(), true);
        let next = collapsed_navigation(activate.clone(), current, layout, variants.clone(), false);
        let child_variants = variants.clone();
        let children = for_index(items, move |index, item| {
            step_row(
                index,
                item,
                current,
                layout,
                activate.clone(),
                child_variants.clone(),
            )
        });
        crate::layer::with_base_recipe_style(
            root.node_ref(reference),
            |r| r.stepper_recipes().root_recipe(),
            &variants,
            move || layout.get(),
        )
        .style(move || {
            style().direction(if orientation.get() == Orientation::Vertical {
                Direction::Column
            } else {
                Direction::Row
            })
        })
        .child(previous)
        .child(children)
        .child(next)
        .into_view()
    }
}

fn responsive_layout(
    items: Value<Vec<StepperItem>>,
    current: Value<usize>,
    orientation: Value<Orientation>,
    min_step_width: Option<Value<f64>>,
    reference: kivact_core::view::NodeRef,
) -> Value<StepperState> {
    let rect = reference.rect();
    memo(move || {
        let count = items.with(Vec::len);
        let minimum = min_step_width.map_or_else(
            || {
                kivact_core::style::use_theme()
                    .get()
                    .ext::<crate::UiTokens>()
                    .stepper
                    .min_step_width
            },
            |width| width.get(),
        );
        StepperState {
            current: current.get(),
            count,
            orientation: orientation.get(),
            collapsed: orientation.get() == Orientation::Horizontal
                && count > 0
                && rect
                    .get()
                    .is_some_and(|rect| rect.width / (count as f64) < minimum),
        }
    })
}
fn collapsed_navigation(
    activate: Option<Rc<dyn Fn(usize)>>,
    current: Value<usize>,
    layout: Value<StepperState>,
    variants: crate::variants::Variants,
    previous: bool,
) -> Option<View> {
    let recipe: fn(&crate::UiRecipes) -> &crate::Recipe<StepperState> = if previous {
        |r| r.stepper_recipes().previous_recipe()
    } else {
        |r| r.stepper_recipes().next_recipe()
    };
    activate.map(|activate| {
        show(
            move || layout.get().collapsed,
            move || {
                let activate = activate.clone();
                let label = if previous {
                    kivact_core::i18n::message("stepper.previous", "Previous step")
                } else {
                    kivact_core::i18n::message("stepper.next", "Next step")
                };
                crate::layer::with_recipe_style(
                    crate::button(label)
                        .disabled(move || {
                            if previous {
                                current.get() == 0
                            } else {
                                current.get().saturating_add(1) >= layout.get().count
                            }
                        })
                        .on_press(move || {
                            activate(if previous {
                                current.get_untracked().saturating_sub(1)
                            } else {
                                current.get_untracked().saturating_add(1)
                            })
                        }),
                    recipe,
                    &variants,
                    move || layout.get(),
                )
            },
        )
        .into_view()
    })
}
fn step_row(
    index: usize,
    item: Value<StepperItem>,
    current: Value<usize>,
    layout: Value<StepperState>,
    activate: Option<Rc<dyn Fn(usize)>>,
    variants: crate::variants::Variants,
) -> impl IntoView {
    let state = move || {
        item.with(|item| StepperItemState {
            index,
            current: current.get(),
            status: item.status,
            collapsed: layout.get().collapsed,
            disabled: item.disabled.get(),
            optional: item.optional.get(),
        })
    };
    let part_variants = variants.clone();
    let contents = kivact_core::view::dynamic(item, move |item| {
        step_contents(item, index, current, state, &part_variants)
    });
    let row = node()
        .role(Role::ListItem)
        .hidden(move || layout.get().collapsed && index != current.get())
        .name(move || item.with(StepperItem::label))
        .current_item(move || (index == current.get()).then_some(CurrentItem::Step));
    let row = if let Some(activate) = &activate {
        let activate = activate.clone();
        row.focusable(move || !state().disabled).on_press(move || {
            if !state().disabled {
                activate(index);
            }
        })
    } else {
        row
    };
    crate::layer::with_recipe_style(row, |r| r.stepper_recipes().item_recipe(), &variants, state)
        .child(contents)
}
fn step_contents(
    item: StepperItem,
    index: usize,
    current: Value<usize>,
    state: impl Fn() -> StepperItemState + Copy + 'static,
    variants: &crate::variants::Variants,
) -> Node {
    let indicator = step_indicator(item.indicator.clone(), index, current);
    let indicator = indicator.map(|indicator| {
        crate::layer::with_recipe_style(
            node(),
            |r| r.stepper_recipes().indicator_recipe(),
            variants,
            state,
        )
        .child(indicator)
    });
    let label = crate::layer::with_recipe_style(
        text(item.label.clone()),
        |r| r.stepper_recipes().label_recipe(),
        variants,
        state,
    );
    let (description, extra, end) = step_details(item, state, variants);
    let optional = kivact_core::view::show(
        move || state().optional,
        || text(kivact_core::i18n::message("stepper.optional", "Optional")),
    );
    let body = column((label, description, optional, extra));
    node()
        .style(style().direction(Direction::Row))
        .child(indicator)
        .child(body)
        .child(end)
}
fn step_indicator(indicator: StepIndicator, index: usize, current: Value<usize>) -> Option<View> {
    match indicator {
        StepIndicator::None => None,
        StepIndicator::Custom(content) => Some((content.0)()),
        StepIndicator::Auto => Some(
            text(move || {
                if index < current.get() {
                    "✓".to_owned()
                } else {
                    (index + 1).to_string()
                }
            })
            .into_view(),
        ),
        _ => Some(text((index + 1).to_string()).into_view()),
    }
}
fn step_details(
    item: StepperItem,
    state: impl Fn() -> StepperItemState + Copy + 'static,
    variants: &crate::variants::Variants,
) -> (Option<View>, Option<View>, Option<View>) {
    let description = item.description.map(|description| {
        crate::layer::with_recipe_style(
            text(description),
            |r| r.stepper_recipes().description_recipe(),
            variants,
            state,
        )
        .into_view()
    });
    let extra = item.content.map(|content| {
        crate::layer::with_recipe_style(
            node(),
            |r| r.stepper_recipes().content_recipe(),
            variants,
            state,
        )
        .child((content.0)())
        .into_view()
    });
    let end = item.end_content.map(|content| {
        crate::layer::with_recipe_style(
            node(),
            |r| r.stepper_recipes().end_content_recipe(),
            variants,
            state,
        )
        .child((content.0)())
        .into_view()
    });

    (description, extra, end)
}
