//! Role: Render reactive progress steps with semantic current-step metadata.
use kivact_core::custom::Reactive;
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::style::{Direction, style};
use kivact_core::view::{
    CurrentItem, IntoView, Node, NodeBuilder, Orientation, Role, View, column, for_index, node,
    text,
};
use std::rc::Rc;
/// Semantic status attached to a step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepStatus {
    #[default]
    Default,
    Accent,
    Success,
    Warning,
    Error,
}
/// A step's optional indicator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepIndicator {
    #[default]
    Auto,
    Number,
    None,
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
/// Recipe state for an individual step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepperItemState {
    pub index: usize,
    pub current: usize,
    pub status: StepStatus,
    pub disabled: bool,
    pub optional: bool,
}
/// A progress list whose current step belongs to the application.
pub struct Stepper {
    root: Node,
    items: Value<Vec<StepperItem>>,
    current: Value<usize>,
    orientation: Value<Orientation>,
    activate: Option<Rc<dyn Fn(usize)>>,
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
        variants: crate::variants::Variants::default(),
    }
}
impl Stepper {
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
            variants,
        } = self;
        let child_variants = variants.clone();
        let children = for_index(items, move |index, item| {
            let state = move || {
                item.with(|item| StepperItemState {
                    index,
                    current: current.get(),
                    status: item.status,
                    disabled: item.disabled.get(),
                    optional: item.optional.get(),
                })
            };
            let contents = kivact_core::view::dynamic(item, move |item| {
                let indicator = match item.indicator {
                    StepIndicator::None => None,
                    StepIndicator::Auto if index < current.get() => Some(text("✓").into_view()),
                    _ => Some(text((index + 1).to_string()).into_view()),
                };
                let description = item.description.map(|description| text(description));
                let extra = item.content.map(|content| (content.0)());
                let end = item.end_content.map(|content| (content.0)());
                let body = column((text(item.label), description, extra));
                node().child(indicator).child(body).child(end)
            });
            let row = node()
                .role(Role::ListItem)
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
            crate::layer::with_recipe_style(
                row,
                |r| r.stepper_recipes().item_recipe(),
                &child_variants,
                state,
            )
            .child(contents)
        });
        crate::layer::with_base_recipe_style(
            root,
            |r| r.stepper_recipes().root_recipe(),
            &variants,
            || (),
        )
        .style(move || {
            style().direction(if orientation.get() == Orientation::Vertical {
                Direction::Column
            } else {
                Direction::Row
            })
        })
        .child(children)
        .into_view()
    }
}
