//! Role: Searchable select and multi-select with chips on the combobox interaction.

use crate::Color;
use crate::choice::{Selection, SelectionValue};
use crate::input_group::{
    InputGroupButtonSize, borderless, input_group, input_group_button, input_group_input,
};
use crate::list_popup::{row_style, surface_style};
use crate::theme_access::{field_fill, tokens};
use kivact::control::icon;
use kivact::control::{Node, NodeBuilder, column, node, row, scroll, text};
use kivact::i18n::{message, message_with};
use kivact::interaction::{
    Combobox as Behavior, ListItemState, ListSelection, use_combobox as behavior,
};
use kivact::primitive::{NodeExt, part};
use kivact::style::{Align, Direction, Justify, pct, style};
use kivact::ui::{ButtonVariant, ListItem};
use kivact::view::{
    Handled, IntoText, IntoValue, IntoView, Key, NodeRef, Placement, View, for_each, node_ref,
    overlay, show,
};
use kivact::{Binding, Value, binding, effect, memo, signal};
use std::hash::Hash;
use std::rc::Rc;

type Matcher = Rc<dyn Fn(&str, &str) -> bool>;

/// Whether the option list is open, and whether the typed text filters it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ComboMode {
    Closed,
    /// Open with every option (the field was pressed, or a key opened it).
    Browsing,
    /// Open and narrowed by the text the user typed.
    Filtering,
}

/// A text field with a filtered popup list of options (`Combobox`).
///
/// Created by [`combobox`]: one choice when it edits an `Option<K>`,
/// several shown as chips when it edits a `Vec<K>`.
/// Typing filters the options; the arrow keys move the active option and
/// Enter chooses it, all from `kivact::interaction::use_combobox`, so the focus
/// stays in the field. Give it an accessible name with `.name(..)`; a
/// combobox with `.disabled(..)` is inert: the field, the chips and the side buttons ignore input.
pub struct Combobox<K: Clone + Eq + Hash + 'static> {
    root: Node,
    options: Value<Vec<ListItem<K>>>,
    mode: Selection<K>,
    placeholder: Option<Value<String>>,
    empty: Option<Value<String>>,
    clear: Value<bool>,
    matcher: Option<Matcher>,
}

fn new<K: Clone + Eq + Hash + 'static>(
    options: Value<Vec<ListItem<K>>>,
    mode: Selection<K>,
) -> Combobox<K> {
    Combobox {
        root: node().style(style().direction(Direction::Column).width(pct(100))),
        options,
        mode,
        placeholder: None,
        empty: None,
        clear: false.into_value(),
        matcher: None,
    }
}

/// A combobox over `options` whose choice is written to `value`.
///
/// The type of `value` decides how many it keeps
/// ([`SelectionValue`](crate::SelectionValue)): an `Option<K>` keeps one
/// choice in the field; a `Vec<K>` keeps several and shows them as chips,
/// each with a remove button, and Backspace in the empty search field
/// removes the last chip.
///
/// ```no_run
/// use kivact::ui::ListItem;
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn framework() -> impl IntoView {
///     let value = signal(None::<&'static str>);
///     basic::combobox(
///         vec![ListItem::new("next", "Next.js"), ListItem::new("astro", "Astro")],
///         value,
///     )
///     .placeholder("Select a framework")
///     .name("Framework")
/// }
/// ```
#[track_caller]
pub fn combobox<K: Clone + Eq + Hash + 'static>(
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl SelectionValue<K>,
) -> Combobox<K> {
    new(options.into_value(), value.into_selection())
}

impl<K: Clone + Eq + Hash + 'static> Combobox<K> {
    /// Hint text shown while the field is empty.
    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.placeholder = Some(text.into_text_value());
        self
    }

    /// The text of the list when no option matches (default `No results found.`).
    pub fn empty_text(mut self, text: impl IntoText) -> Self {
        self.empty = Some(text.into_text_value());
        self
    }

    /// Show a button that clears the choice while true (single choice only;
    /// constant or reactive).
    #[track_caller]
    pub fn show_clear(mut self, show: impl IntoValue<bool>) -> Self {
        self.clear = show.into_value();
        self
    }

    /// Decide which options the typed text keeps: `matches(query, label)` is
    /// called with the trimmed text in the field and the option's label. The
    /// default keeps the labels that contain the text, ignoring case.
    pub fn filter(mut self, matches: impl Fn(&str, &str) -> bool + 'static) -> Self {
        self.matcher = Some(Rc::new(matches));
        self
    }
}

impl<K: Clone + Eq + Hash + 'static> NodeBuilder for Combobox<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// The label of `key` among `options`, empty when it is absent.
fn label_of<K: Clone + Eq + Hash + 'static>(options: Value<Vec<ListItem<K>>>, key: &K) -> String {
    options.with(|all| {
        all.iter()
            .find(|o| o.key() == key)
            .map(|o| o.label())
            .unwrap_or_default()
    })
}

/// A small ghost icon button that never takes the focus from the field.
fn side_button(
    name: Value<String>,
    glyph: &'static str,
    input: NodeRef,
    off: Value<bool>,
    on_press: impl Fn() + 'static,
) -> View {
    input_group_button(name)
        .size(InputGroupButtonSize::IconXs)
        .variant(ButtonVariant::Ghost)
        .icon(icon(glyph))
        .focusable(false)
        .disabled(off)
        .on_press(on_press)
        .part(part(|s| s).focus_on_press(input))
        .style(style().color(Color::MutedForeground))
        .into_view()
}

/// The popup of `list`: the options that match, or the empty text.
fn popup<K: Clone + Eq + Hash + 'static>(
    list: Behavior<K>,
    options: Value<Vec<ListItem<K>>>,
    open: Binding<bool>,
    anchor: NodeRef,
    empty: Value<String>,
    name: Value<String>,
    is_chosen: impl Fn(&K) -> bool + Copy + 'static,
) -> View {
    let nothing = memo(move || list.visible().with(Vec::is_empty));
    show(open.into_value(), move || {
        let rows = for_each(
            list.visible(),
            |key| key.clone(),
            move |key, _| {
                let part = list.item(key.clone());
                let state = part.state();
                let label = memo({
                    let key = key.clone();
                    move || label_of(options, &key)
                });
                node()
                    .part(part)
                    .style(move || {
                        let st = ListItemState {
                            selected: is_chosen(&key),
                            ..state.get()
                        };
                        row_style(&tokens(), &st)
                    })
                    .child(text(label))
            },
        );
        let content = column((
            scroll(column(rows))
                .part(list.content())
                .name(name)
                .style(style().max_height(252)),
            show(nothing, move || {
                text(empty).style(
                    style()
                        .width(pct(100))
                        .padding_y(8)
                        .font_size(12)
                        .text_align(kivact::style::TextAlign::Center)
                        .color(Color::MutedForeground),
                )
            }),
        ))
        .style(move || surface_style(&tokens()).max_height(288));
        overlay(anchor, Placement::Below, content)
            .offset(6.0)
            .at_least_anchor_width(true)
            .on_dismiss(move |_| open.set(false))
    })
    .into_view()
}

/// The chips field of the multiple combobox (`ComboboxChips`).
fn chips_field<K: Clone + Eq + Hash + 'static>(
    options: Value<Vec<ListItem<K>>>,
    input: kivact::ui::TextInput,
    values: Binding<Vec<K>>,
    input_ref: NodeRef,
    anchor: NodeRef,
    off: Value<bool>,
) -> View {
    let within = anchor.focus_within();
    let chips = for_each(
        values.into_value(),
        |key| key.clone(),
        move |key, _| {
            let label = memo({
                let key = key.clone();
                move || label_of(options, &key)
            });
            let remove_name = message_with("basic.combobox.remove", "Remove {label}", move || {
                [("label", label.get())]
            });
            let remove = node()
                .role(kivact::view::Role::Button)
                .name(remove_name)
                .disabled(off)
                .child(icon("x"))
                .part(part(|s| s).focus_on_press(input_ref))
                .on_press(move || {
                    values.set(values.get().into_iter().filter(|k| k != &key).collect());
                })
                .style(
                    style()
                        .size(16, 16)
                        .opacity(0.5)
                        .hover(style().opacity(1.0)),
                );
            row((text(label), remove)).style(
                style()
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .gap(4)
                    .height(21)
                    .padding_left(6)
                    .padding_right(2)
                    .radius(0)
                    .background(Color::Muted)
                    .color(Color::Foreground)
                    .font_size(12)
                    .weight(500)
                    .text_wrap(false),
            )
        },
    );
    row((
        chips,
        borderless(input, Some(22.0)).style(style().min_width(64).padding_x(2)),
    ))
    .node_ref(anchor)
    .part(part(|s| s).focus_on_press(input_ref))
    .style(move || {
        let p = tokens();
        let s = style()
            .direction(Direction::Row)
            .wrap(true)
            .align_items(Align::Center)
            .gap(4)
            .width(pct(100))
            .min_height(32)
            .padding_x(4)
            .padding_y(4)
            .radius(0)
            .background(field_fill(&p))
            .border(1, p.input);
        let s = if off.get() {
            // `has-disabled:bg-input/50 has-disabled:opacity-50`, `/80` in the dark.
            s.merge(&crate::theme_access::disabled())
                .background(p.input.with_alpha(if p.is_dark() { 0.8 } else { 0.5 }))
        } else {
            s
        };
        // The field inside has the focus: the box is not a focus stop and
        // has no state style for it, so it follows `within`.
        if within.get() {
            s.merge(&crate::theme_access::focus_ring(&p))
        } else {
            s
        }
    })
    .into_view()
}

impl<K: Clone + Eq + Hash + 'static> IntoView for Combobox<K> {
    fn into_view(mut self) -> View {
        let name_source = self.root.accessible_name();
        let fallback = message("basic.combobox.label", "Combobox");
        let name: Value<String> = memo(move || {
            name_source
                .as_ref()
                .map(|read| read())
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| fallback.get())
        });
        let Combobox {
            mut root,
            options,
            mode,
            placeholder,
            empty,
            clear,
            matcher,
        } = self;
        let disabled: Option<Value<bool>> = root
            .accessible_disabled()
            .map(|read| (move || read()).into_value());
        let off = memo(move || disabled.is_some_and(|d| d.get()));
        let (single_value, multi_value) = match mode {
            Selection::Single(v) => (Some(v), None),
            Selection::Multiple(v) => (None, Some(v)),
        };
        let mode = signal(ComboMode::Closed);
        // The list is open in either mode; only `Filtering` narrows the options.
        let open = binding(
            memo(move || mode.get() != ComboMode::Closed),
            move |open: bool| {
                mode.update(|mode| match (open, *mode) {
                    (false, _) => *mode = ComboMode::Closed,
                    (true, ComboMode::Closed) => *mode = ComboMode::Browsing,
                    _ => {}
                });
            },
        );
        let single: Value<Option<K>> = match single_value {
            Some(v) => v.into_value(),
            None => (|| None::<K>).into_value(),
        };
        // A single choice is written by the behavior, which also closes the
        // list; several are toggled in their own list.
        let chosen: Binding<Option<K>> = match single_value {
            Some(v) => v,
            None => binding(single, |_: Option<K>| {}),
        };
        let selection = match multi_value {
            Some(values) => ListSelection::Multiple(values),
            None => ListSelection::Single(chosen),
        };
        let list = behavior(options, selection).popup(open);
        let query = list.query();
        let list = list.filter(move |q: &str, item: &ListItem<K>| {
            if mode.get() != ComboMode::Filtering {
                return true;
            }
            let label = item.label();
            match &matcher {
                Some(matches) => matches(q.trim(), &label),
                None => label.to_lowercase().contains(&q.trim().to_lowercase()),
            }
        });
        // While the list is closed the field shows the chosen label.
        effect(move || {
            let closed = mode.get() == ComboMode::Closed;
            let chosen = single.get();
            if closed {
                query.set(
                    chosen
                        .as_ref()
                        .map(|k| label_of(options, k))
                        .unwrap_or_default(),
                );
            }
        });
        let edit = binding(query, move |s: String| {
            query.set(s);
            mode.set(ComboMode::Filtering);
        });
        let input_part = list.input();
        let input_ref = input_part.node_ref();
        let mut field_input = input_group_input(name, edit)
            .forward_keys(vec![Key::ArrowDown, Key::ArrowUp, Key::Enter, Key::Escape])
            .part(input_part)
            .expanded(open.into_value())
            .on_pointer(move |event: kivact::view::PointerEvent| {
                if matches!(event.phase, kivact::view::PointerPhase::Down(_))
                    && !off.get()
                    && mode.get_untracked() == ComboMode::Closed
                {
                    mode.set(ComboMode::Browsing);
                }
                Handled::No
            });
        if let Some(hint) = placeholder {
            field_input = field_input.placeholder(hint);
        }
        if let Some(disabled) = disabled {
            field_input = field_input.disabled(disabled);
        }
        let anchor = node_ref();
        let body: View = if let Some(values) = multi_value {
            chips_field(options, field_input, values, input_ref, anchor, off)
        } else {
            let chevron = side_button(
                message("basic.combobox.toggle", "Toggle options"),
                "chevron-down",
                input_ref,
                off,
                move || {
                    mode.update(|mode| {
                        *mode = if *mode == ComboMode::Closed {
                            ComboMode::Browsing
                        } else {
                            ComboMode::Closed
                        };
                    });
                },
            );
            let clear_name = message("basic.combobox.clear", "Clear");
            let clear_button = show(
                memo(move || clear.get() && single.with(Option::is_some)),
                move || {
                    side_button(clear_name, "x", input_ref, off, move || {
                        if let Some(value) = single_value {
                            value.set(None);
                        }
                    })
                },
            );
            input_group(field_input)
                .end(row((clear_button, chevron)).style(style().align_items(Align::Center)))
                .anchored(anchor)
                .disabled(off)
                .into_view()
        };
        let is_chosen = move |key: &K| match (single_value, multi_value) {
            (Some(v), _) => v.with(|chosen| chosen.as_ref() == Some(key)),
            (_, Some(v)) => v.with(|chosen| chosen.contains(key)),
            _ => false,
        };
        let empty = empty.unwrap_or_else(|| message("basic.combobox.empty", "No results found."));
        let rows = popup(list, options, open, anchor, empty, name, is_chosen);
        root.child(body).child(rows).into_view()
    }
}
