//! Role: The stepper builder, its items, indicators, responsive layout and navigation.

use crate::stepper::look::StepperTheme;
use crate::stepper::look::{StepIndicatorKind, StepStatus, StepperItemState, StepperState};
use kivact_core::custom::Prop;
use kivact_core::state::{Callback, IntoText, IntoValue, Signal, Value, callback, memo};
use kivact_core::style::{FlexDirection, style};
use kivact_core::view::{
    CurrentItem, EditPhase, Element, IntoView, Node, Orientation, Role, View, column, for_index,
    node, node_ref, show, text,
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
    label: Prop<String>,
    description: Option<Prop<String>>,
    status: StepStatus,
    disabled: Prop<bool>,
    optional: Prop<bool>,
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
    #[track_caller]
    pub fn new(label: impl IntoText) -> Self {
        Self {
            label: Prop::new(label.into_text()),
            description: None,
            status: StepStatus::Default,
            disabled: Prop::new(false),
            optional: Prop::new(false),
            indicator: StepIndicator::Auto,
            content: None,
            end_content: None,
        }
    }
    /// Add supporting text beneath the label.
    #[track_caller]
    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(Prop::new(description.into_text()));
        self
    }
    /// Set semantic status for recipes and assistive descriptions.
    pub fn status(mut self, status: StepStatus) -> Self {
        self.status = status;
        self
    }
    /// Prevent activation of this step.
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = Prop::new(disabled);
        self
    }
    /// Mark a step as optional.
    pub fn optional(mut self, optional: impl IntoValue<bool>) -> Self {
        self.optional = Prop::new(optional);
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
    current: Signal<usize>,
    orientation: Value<Orientation>,
    collapsed_steps: CollapsedSteps,
    on_edit: Option<Callback<EditPhase>>,
    min_step_width: Option<Value<f64>>,
    variants: crate::variants::Variants,
}
/// Show progress through an ordered sequence of steps. A press on a step
/// (or on the collapsed previous and next buttons) writes its index to
/// `current`, and `on_edit` reports the write as an edit. Every step is
/// pressable; give `StepperItem::disabled(true)` to a step the user may not
/// jump to (a linear flow).
#[track_caller]
pub fn stepper(items: impl IntoValue<Vec<StepperItem>>, current: Signal<usize>) -> Stepper {
    Stepper {
        root: node()
            .role(Role::List)
            .name(kivact_core::i18n::message("stepper.label", "Progress")),
        items: items.into_value(),
        current,
        orientation: Orientation::Horizontal.into_value(),
        collapsed_steps: CollapsedSteps::Current,
        on_edit: None,
        min_step_width: None,
        variants: crate::variants::Variants::default(),
    }
}
impl Stepper {
    /// Override the minimum width used to derive the collapsed layout.
    #[track_caller]
    pub fn min_step_width(mut self, width: impl IntoValue<f64>) -> Self {
        self.min_step_width = Some(width.into_value());
        self
    }
    /// Choose the compact structure without changing current-step ownership.
    pub fn collapsed_steps(mut self, steps: CollapsedSteps) -> Self {
        self.collapsed_steps = steps;
        self
    }
    /// Choose horizontal or vertical layout.
    #[track_caller]
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        self.orientation = orientation.into_value();
        self
    }
    /// Report each [`EditPhase`] of a write to `current` by the user: a
    /// press on a step, or on the collapsed previous and next buttons. A
    /// write from the application own `set` is not reported.
    pub fn on_edit<R: kivact_core::view::HandlerResult>(
        mut self,
        f: impl Fn(EditPhase) -> R + 'static,
    ) -> Self {
        self.on_edit = Some(callback(f));
        self
    }
    /// Add kit-defined visual metadata.
    pub fn variant(mut self, variant: impl crate::theme::Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl Element for Stepper {
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
            collapsed_steps,
            on_edit,
            min_step_width,
            variants,
        } = self;
        let edit = kivact_interaction::EditSession::new(move |initial: usize| current.set(initial));
        let edit = match on_edit {
            Some(on_edit) => edit.on_edit(move |phase| on_edit.call(phase)),
            None => edit,
        };
        // Every user write of the index goes through here: it clamps the
        // index to the items and reports one edit when it changes the value.
        let write = callback(move |index: usize| {
            let index = index.min(items.with_untracked(|items| items.len().saturating_sub(1)));
            if index != current.get_untracked() {
                edit.once(current.get_untracked(), move || current.set(index));
            }
        });
        let reference = node_ref();
        let layout = responsive_layout(items, current, orientation, min_step_width, reference);
        let previous = collapsed_navigation(write, items, current, layout, variants.clone(), true);
        let next = collapsed_navigation(write, items, current, layout, variants.clone(), false);
        let child_variants = variants.clone();
        let children = for_index(items, move |index, item| {
            step_row(
                index,
                item,
                current,
                layout,
                collapsed_steps,
                write,
                child_variants.clone(),
            )
        });
        crate::layer::with_base_recipe_style(
            root.node_ref(reference),
            |r: &StepperTheme| r.root_recipe(),
            &variants,
            move || layout.get(),
        )
        .style(move || {
            style().flex_direction(if orientation.get() == Orientation::Vertical {
                FlexDirection::Column
            } else {
                FlexDirection::Row
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
    current: Signal<usize>,
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
                    .get(&crate::stepper::look::MIN_STEP_WIDTH_VAR)
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
    write: Callback<usize>,
    items: Value<Vec<StepperItem>>,
    current: Signal<usize>,
    layout: Value<StepperState>,
    variants: crate::variants::Variants,
    previous: bool,
) -> View {
    let recipe: fn(&StepperTheme) -> &crate::recipe::Recipe<StepperState> = if previous {
        |r: &StepperTheme| r.previous_recipe()
    } else {
        |r: &StepperTheme| r.next_recipe()
    };
    let target = memo(move || {
        items.with(|items| {
            let current = current.get();
            if previous {
                (0..current.min(items.len()))
                    .rev()
                    .find(|at| !items[*at].disabled.get())
            } else {
                (current.saturating_add(1)..items.len()).find(|at| !items[*at].disabled.get())
            }
        })
    });
    show(
        move || layout.get().collapsed,
        move || {
            let label = if previous {
                kivact_core::i18n::message("stepper.previous", "Previous step")
            } else {
                kivact_core::i18n::message("stepper.next", "Next step")
            };
            crate::layer::with_recipe_style(
                crate::button(label)
                    .disabled(move || target.get().is_none())
                    .on_press(move || {
                        if let Some(target) = target.get_untracked() {
                            write.call(target);
                        }
                    }),
                recipe,
                &variants,
                move || layout.get(),
            )
        },
    )
    .into_view()
}
fn step_row(
    index: usize,
    item: Value<StepperItem>,
    current: Signal<usize>,
    layout: Value<StepperState>,
    collapsed_steps: CollapsedSteps,
    write: Callback<usize>,
    variants: crate::variants::Variants,
) -> impl IntoView {
    let state = move || {
        item.with(|item| StepperItemState {
            count: layout.get().count,
            orientation: layout.get().orientation,
            index,
            current: current.get(),
            status: item.status,
            indicator: match item.indicator {
                StepIndicator::Auto => StepIndicatorKind::Auto,
                StepIndicator::Number => StepIndicatorKind::Number,
                StepIndicator::None => StepIndicatorKind::None,
                StepIndicator::Custom(_) => StepIndicatorKind::Custom,
            },
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
        .hidden(move || {
            layout.get().collapsed
                && collapsed_steps == CollapsedSteps::Current
                && index != current.get()
        })
        .name(move || item.with(StepperItem::label))
        .current_item(move || (index == current.get()).then_some(CurrentItem::Step));
    let row = row.focusable(move || !state().disabled).on_press(move || {
        if !state().disabled {
            write.call(index);
        }
    });
    crate::layer::with_recipe_style(row, |r: &StepperTheme| r.item_recipe(), &variants, state)
        .child(contents)
}
fn step_contents(
    item: StepperItem,
    index: usize,
    current: Signal<usize>,
    state: impl Fn() -> StepperItemState + Copy + 'static,
    variants: &crate::variants::Variants,
) -> Node {
    let indicator = step_indicator(item.indicator.clone(), index, current);
    let indicator = indicator.map(|indicator| {
        crate::layer::with_recipe_style(
            node(),
            |r: &StepperTheme| r.indicator_recipe(),
            variants,
            state,
        )
        .child(indicator)
    });
    let label = crate::layer::with_recipe_style(
        text({
            let label = item.label.clone();
            move || label.get()
        }),
        |r: &StepperTheme| r.label_recipe(),
        variants,
        state,
    );
    let (description, extra, end) = step_details(item, state, variants);
    let optional = kivact_core::view::show(
        move || state().optional,
        || text(kivact_core::i18n::message("stepper.optional", "Optional")),
    );
    let body = crate::layer::with_recipe_style(
        column((label, description, optional, extra)),
        |r: &StepperTheme| r.body_recipe(),
        variants,
        state,
    );
    let track = crate::layer::with_recipe_style(
        node(),
        |r: &StepperTheme| r.track_recipe(),
        variants,
        state,
    )
    .child((
        step_decoration(variants, state, StepDecoration::Leading),
        indicator,
        step_decoration(variants, state, StepDecoration::Trailing),
    ));
    crate::layer::with_recipe_style(
        node(),
        |r: &StepperTheme| r.layout_recipe(),
        variants,
        state,
    )
    .child((
        track,
        body,
        end,
        step_decoration(variants, state, StepDecoration::Progress),
    ))
}
fn step_indicator(indicator: StepIndicator, index: usize, current: Signal<usize>) -> Option<View> {
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
            text(move || description.get()),
            |r: &StepperTheme| r.description_recipe(),
            variants,
            state,
        )
        .into_view()
    });
    let extra = item.content.map(|content| {
        crate::layer::with_recipe_style(
            node(),
            |r: &StepperTheme| r.content_recipe(),
            variants,
            state,
        )
        .child((content.0)())
        .into_view()
    });
    let end = item.end_content.map(|content| {
        crate::layer::with_recipe_style(
            node(),
            |r: &StepperTheme| r.end_content_recipe(),
            variants,
            state,
        )
        .child((content.0)())
        .into_view()
    });

    (description, extra, end)
}

#[derive(Clone, Copy)]
enum StepDecoration {
    Leading,
    Trailing,
    Progress,
}
/// Mount only decorative geometry; selection and compact layout remain with the existing source.
fn step_decoration(
    variants: &crate::variants::Variants,
    state: impl Fn() -> StepperItemState + 'static,
    part: StepDecoration,
) -> Node {
    let pick: fn(&StepperTheme) -> &crate::recipe::Recipe<StepperItemState> = match part {
        StepDecoration::Leading => |r: &StepperTheme| r.leading_recipe(),
        StepDecoration::Trailing => |r: &StepperTheme| r.trailing_recipe(),
        StepDecoration::Progress => |r: &StepperTheme| r.progress_recipe(),
    };
    crate::layer::with_recipe_style(
        node().hit_testable(false).focusable(false),
        pick,
        variants,
        state,
    )
}

/// Which steps remain mounted visibly when the responsive layout is compact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CollapsedSteps {
    #[default]
    Current,
    All,
}
