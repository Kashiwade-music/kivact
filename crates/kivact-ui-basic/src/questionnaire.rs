//! Role: Step-by-step questionnaire with choices or text input and progress.

use crate::theme_access::tokens;
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::i18n::{message, message_with};
use kivact::interaction::{ListItemState, ListPolicy, ListSelection, use_list};
use kivact::style::{Align, Color, Direction, Justify, Position, pct, style};
use kivact::ui::{ButtonVariant, ListItem, button, text_input};
use kivact::view::HandlerResult;
use kivact::view::{IntoValue, IntoView, Role, View, callback, dynamic, for_each};
use kivact::{Binding, Callback, Value, binding, memo};

/// One choice of a [`QuestionnaireStep`].
#[derive(Clone, Debug, PartialEq)]
pub struct QuestionnaireChoice {
    value: String,
    label: String,
    description: Option<String>,
}

impl QuestionnaireChoice {
    /// A choice whose answer is `value`, shown as `label`.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
        }
    }

    /// A line of helper text under the label.
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
enum StepKind {
    Choices(Vec<QuestionnaireChoice>),
    Input { placeholder: Option<String> },
}

/// One question of a [`Questionnaire`].
#[derive(Clone, Debug, PartialEq)]
pub struct QuestionnaireStep {
    title: String,
    description: Option<String>,
    kind: StepKind,
    optional: bool,
}

impl QuestionnaireStep {
    /// A question answered by one of `choices`.
    pub fn choices(title: impl Into<String>, choices: Vec<QuestionnaireChoice>) -> Self {
        Self {
            title: title.into(),
            description: None,
            kind: StepKind::Choices(choices),
            optional: false,
        }
    }

    /// A question answered with free text.
    pub fn input(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            kind: StepKind::Input { placeholder: None },
            optional: false,
        }
    }

    /// A line of helper text under the title.
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// Hint text of a text question.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        if let StepKind::Input { placeholder } = &mut self.kind {
            *placeholder = Some(text.into());
        }
        self
    }

    /// Let the user skip the question (default: it must be answered). A
    /// constant: a step is plain data inside the `steps` value, so a changed
    /// flag is a new list.
    pub fn optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }
}

/// A questionnaire that asks one question at a time (`Questionnaire`).
///
/// Created by [`questionnaire`]. The step shown is the caller's `current`
/// binding, and the answers are the caller's `answers` binding, one
/// `Option<String>` per step (`None` is unanswered or skipped). It writes a
/// whole list into that binding, and the list grows as the steps are
/// answered: a questionnaire is a form whose result is its answer sheet, so
/// the application can read it back, restore it or reset it.
/// The root is a `Role::Group` named by the message `questionnaire.label`
/// ("Questionnaire") unless the caller names it. The texts are messages:
/// `questionnaire.previous`, `questionnaire.skip`, `questionnaire.next`,
/// `questionnaire.submit` and `questionnaire.progress`.
pub struct Questionnaire {
    root: Node,
    steps: Value<Vec<QuestionnaireStep>>,
    current: Binding<usize>,
    answers: Binding<Vec<Option<String>>>,
    submit: Option<Callback<Vec<Option<String>>>>,
}

/// A questionnaire over `steps` whose shown step is `current` (zero-based)
/// and whose answers are `answers`.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn survey() -> impl IntoView {
///     let step = signal(0usize);
///     let answers = signal(Vec::new());
///     let steps = vec![basic::QuestionnaireStep::choices(
///         "How did you hear about us?",
///         vec![
///             basic::QuestionnaireChoice::new("search", "Search"),
///             basic::QuestionnaireChoice::new("friend", "A friend"),
///         ],
///     )];
///     basic::questionnaire(steps, step, answers).on_submit(|answers: Vec<Option<String>>| {
///         println!("{answers:?}");
///     })
/// }
/// ```
#[track_caller]
pub fn questionnaire(
    steps: impl IntoValue<Vec<QuestionnaireStep>>,
    current: impl Into<Binding<usize>>,
    answers: impl Into<Binding<Vec<Option<String>>>>,
) -> Questionnaire {
    Questionnaire {
        root: node()
            .role(Role::Group)
            .name(message("basic.questionnaire.label", "Questionnaire"))
            .style(
                style()
                    .direction(Direction::Column)
                    .gap(16)
                    .width(pct(100))
                    .min_width(0)
                    .shrink(0),
            ),
        steps: steps.into_value(),
        current: current.into(),
        answers: answers.into(),
        submit: None,
    }
}

impl Questionnaire {
    /// Called with the answers when the last step is submitted.
    pub fn on_submit<R: HandlerResult>(
        mut self,
        f: impl Fn(Vec<Option<String>>) -> R + 'static,
    ) -> Self {
        self.submit = Some(callback(f));
        self
    }
}

impl NodeBuilder for Questionnaire {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// Write the answer of step `at`, growing the answer list.
fn set_answer(answers: Binding<Vec<Option<String>>>, at: usize, value: Option<String>) {
    let mut all = answers.get();
    if all.len() <= at {
        all.resize(at + 1, None);
    }
    all[at] = value;
    answers.set(all);
}

/// The answer of step `at` as a binding that grows the answer list.
fn answer_of(answers: Binding<Vec<Option<String>>>, at: usize) -> Binding<Option<String>> {
    binding(
        move || answers.with(|a| a.get(at).cloned().flatten()),
        move |value: Option<String>| set_answer(answers, at, value),
    )
}

/// A choice card: `min-h-11 border-input px-3 py-2.5`, muted when chosen.
fn choice_card(choice: &QuestionnaireChoice, part: kivact::primitive::Part<ListItemState>) -> Node {
    let state = part.state();
    let mut body = vec![
        text(choice.label.clone())
            .style(move || style().font_size(12).line_height(1.375))
            .into_view(),
    ];
    if let Some(description) = &choice.description {
        body.push(
            text(description.clone())
                .style(
                    style()
                        .font_size(12)
                        .line_height(1.375)
                        .color(crate::Color::MutedForeground),
                )
                .into_view(),
        );
    }
    let indicator = node()
        .child(node().style(move || {
            let p = tokens();
            style()
                .position(Position::Absolute)
                .left(3)
                .top(3)
                .size(8, 8)
                .radius(4)
                .background(if state.get().selected {
                    p.primary_foreground
                } else {
                    Color::TRANSPARENT
                })
        }))
        .style(move || {
            let p = tokens();
            let selected = state.get().selected;
            style()
                .position(Position::Relative)
                .size(16, 16)
                .shrink(0)
                .margin_top(2)
                .radius(8)
                .border(1, if selected { p.primary } else { p.input })
                .background(if selected {
                    p.primary
                } else {
                    Color::TRANSPARENT
                })
        });
    node()
        .part(part)
        .role(Role::RadioButton)
        .child(indicator)
        .child(column(body).style(style().gap(2).grow(1).min_width(0)))
        .style(move || {
            let p = tokens();
            let st = state.get();
            let s = style()
                .direction(Direction::Row)
                .align_items(Align::Start)
                .gap(10)
                .min_height(44)
                .padding_x(12)
                .padding_y(10)
                .radius(0)
                .font_size(12)
                .color(p.foreground)
                .border(
                    1,
                    if st.selected {
                        p.foreground.with_alpha(0.3)
                    } else {
                        p.input
                    },
                )
                .background(if st.selected {
                    p.muted
                } else {
                    Color::TRANSPARENT
                })
                .focus_visible(crate::theme_access::focus_ring(&p));
            if st.selected {
                s
            } else {
                s.hover(style().background(p.muted.with_alpha(0.5)))
            }
        })
}

/// The question of one step: its title, description and answer control.
fn question(step: &QuestionnaireStep, answer: Binding<Option<String>>) -> Node {
    let title = step.title.clone();
    let mut parts = vec![
        text(title.clone())
            .role(Role::Heading)
            .style(style().font_size(14).line_height(20.0 / 14.0).weight(500))
            .into_view(),
    ];
    if let Some(description) = &step.description {
        parts.push(
            text(description.clone())
                .style(
                    style()
                        .font_size(12)
                        .line_height(1.625)
                        .color(crate::Color::MutedForeground),
                )
                .into_view(),
        );
    }
    match &step.kind {
        StepKind::Choices(choices) => {
            let items: Vec<ListItem<String>> = choices
                .iter()
                .map(|c| ListItem::new(c.value.clone(), c.label.clone()))
                .collect();
            let items = items.into_value();
            let group = use_list(items, ListSelection::Single(answer), ListPolicy::radio());
            let choices = choices.clone();
            let rows = for_each(
                items,
                |item| item.key().clone(),
                move |key, _| {
                    let choice = choices
                        .iter()
                        .find(|c| c.value == key)
                        .cloned()
                        .unwrap_or_else(|| QuestionnaireChoice::new(key.clone(), key.clone()));
                    choice_card(&choice, group.item(key))
                },
            );
            parts.push(
                column(rows)
                    .role(Role::RadioGroup)
                    .name(title)
                    .style(style().gap(8).margin_top(if step.description.is_some() {
                        0
                    } else {
                        16
                    }))
                    .into_view(),
            );
        }
        StepKind::Input { placeholder } => {
            let text_value = binding(
                move || answer.with(|a| a.clone().unwrap_or_default()),
                move |s: String| answer.set((!s.is_empty()).then_some(s)),
            );
            let mut input = text_input(title, text_value).style(style().min_height(32));
            if let Some(hint) = placeholder {
                input = input.placeholder(hint.clone());
            }
            parts.push(input.into_view());
        }
    }
    column(parts).style(style().gap(if step.description.is_some() { 8 } else { 0 }))
}

impl IntoView for Questionnaire {
    fn into_view(self) -> View {
        let Questionnaire {
            root,
            steps,
            current,
            answers,
            submit,
        } = self;
        let total = memo(move || steps.with(Vec::len));
        let index = memo(move || current.get().min(total.get().saturating_sub(1)));
        let progress = message_with(
            "basic.questionnaire.progress",
            "{current} of {total}",
            move || {
                [
                    ("current", (index.get() + 1).to_string()),
                    ("total", total.get().to_string()),
                ]
            },
        );
        let step_view = dynamic(
            move || index.get(),
            move |at| {
                steps
                    .with(|all| all.get(at).cloned())
                    .map(|step| question(&step, answer_of(answers, at)))
                    .unwrap_or_else(node)
            },
        );
        let answered =
            memo(move || answers.with(|a| a.get(index.get()).is_some_and(Option::is_some)));
        let optional =
            memo(move || steps.with(|all| all.get(index.get()).is_some_and(|s| s.optional)));
        let first = memo(move || index.get() == 0);
        let last = memo(move || index.get() + 1 >= total.get());
        let finish = move || {
            if let Some(submit) = submit {
                submit.call(answers.get_untracked());
            }
        };
        let previous = kivact::view::show(memo(move || !first.get()), move || {
            button(message("basic.questionnaire.previous", "Previous"))
                .variant(ButtonVariant::Outline)
                .on_press(move || current.set(index.get().saturating_sub(1)))
                .style(style().align_self(Align::Start))
        });
        let skip = kivact::view::show(optional, move || {
            button(message("basic.questionnaire.skip", "Skip"))
                .variant(ButtonVariant::Outline)
                .on_press(move || {
                    set_answer(answers, index.get(), None);
                    if last.get() {
                        finish();
                    } else {
                        current.set(index.get() + 1);
                    }
                })
        });
        let forward = dynamic(
            move || last.get(),
            move |is_last| {
                let blocked = memo(move || !answered.get() && !optional.get());
                if is_last {
                    button(message("basic.questionnaire.submit", "Submit"))
                        .disabled(blocked)
                        .on_press(finish)
                } else {
                    button(message("basic.questionnaire.next", "Next"))
                        .disabled(blocked)
                        .on_press(move || current.set(index.get() + 1))
                }
            },
        );
        let actions = row((node().style(style().grow(1)).child(previous), skip, forward)).style(
            style()
                .align_items(Align::Center)
                .justify_content(Justify::Start)
                .gap(6)
                .min_height(32),
        );
        root.child(
            text(progress).style(
                style()
                    .font_size(12)
                    .weight(500)
                    .min_width(100)
                    .tabular_nums()
                    .color(crate::Color::MutedForeground),
            ),
        )
        .child(step_view)
        .child(actions)
        .into_view()
    }
}
