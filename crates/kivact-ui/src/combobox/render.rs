//! Role: Render one combobox query and selection through themed editor and list parts.
use super::ComboboxMode;
use super::data::Combobox;
use super::labels::label;
use super::notices::Notices;
use crate::combobox::look::ComboboxTheme;
use crate::helpers::{is_named, report_unnamed};
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::{ButtonVariant, button, text_input};
use kivact_core::custom::{PartFacts, PartState};
use kivact_core::state::{IntoValue, ItemKey, Value, binding, memo, signal};
use kivact_core::view::{
    Element, Handled, IntoView, Key, Placement, PointerKind, PointerPhase, View, column, for_each,
    icon, node, node_ref, overlay, row, scroll, show, text,
};
use kivact_interaction::{
    ComboboxApi, ComboboxOptions, ListItem, Selection, known_options, use_combobox,
};

/// The combobox frame takes no input while the combobox is disabled.
fn frame_availability(disabled: bool) -> kivact_core::view::Availability {
    if disabled {
        kivact_core::view::Availability::Disabled
    } else {
        kivact_core::view::Availability::default()
    }
}

/// The combobox frame rings while the focus is within it, and takes no input
/// while the combobox is disabled.
fn frame_state(within: bool, disabled: bool) -> PartState {
    PartState::fixed(PartFacts {
        focus: if within {
            kivact_core::view::Focus::Focused
        } else {
            kivact_core::view::Focus::None
        },
        availability: frame_availability(disabled),
    })
}

pub(super) fn render<K: ItemKey>(mut field: Combobox<K>) -> View {
    let mounted = prepare(&mut field);
    let Mounted {
        list,
        selection,
        known,
        notices,
        name,
        disabled,
        invalid,
        inline,
    } = mounted;
    let Combobox {
        mut root,
        free_text,
        chips,
        placeholder,
        empty,
        clear,
        presentation,
        ..
    } = field;
    let variants = root.authored().variants();
    let input_ref = node_ref();
    let anchor = node_ref();
    let within = anchor.focus_within();
    let placeholder =
        placeholder.unwrap_or_else(|| kivact_core::i18n::message("combobox.placeholder", "Search"));
    let editor = editor_view(
        EditorState {
            list,
            selection,
            known,
            name,
            disabled,
            invalid,
        },
        input_ref,
        free_text,
        placeholder,
        presentation,
        &variants,
        &mut root,
    );
    let tokens = selected_chips(chips, selection, known, disabled, input_ref, &variants);
    let (clear_button, toggle) =
        editor_actions(list, selection, clear, disabled, input_ref, inline);
    let frame = with_base_recipe_style(
        row((editor, tokens, clear_button, toggle))
            .node_ref(anchor)
            .with_node(move |node| node.focus_on_press(input_ref)),
        |r: &ComboboxTheme| r.frame_recipe(),
        &variants,
        move || frame_state(within.get(), disabled.get()),
    );
    let empty =
        empty.unwrap_or_else(|| kivact_core::i18n::message("combobox.empty", "No results found."));
    let loading = kivact_core::i18n::message("combobox.loading", "Loading…");
    let failed = kivact_core::i18n::message("combobox.failed", "Could not load suggestions.");
    let search_status = list.search_status();
    let empty = memo(move || match search_status.get() {
        kivact_core::task::AsyncState::Pending => loading.get(),
        kivact_core::task::AsyncState::Failed(_) => failed.get(),
        _ => empty.get(),
    });
    let source = list.options();
    let surface = popup_surface(list, source, name, empty, variants.clone(), anchor, inline);
    let sheet = sheet_surface(list, source, name, empty, variants, disabled, placeholder);
    root.child(frame)
        .child(surface)
        .child(sheet)
        .child(notices.outlet())
        .into_view()
}
fn suggestions<K: ItemKey>(
    list: ComboboxApi<K>,
    source: Value<Vec<ListItem<K>>>,
    name: Value<String>,
    empty: Value<String>,
    variants: kivact_core::view::Variants,
    input: Option<kivact_core::view::NodeRef>,
) -> View {
    let rows = for_each(list.visible(), |key| key.clone(), {
        let variants = variants.clone();
        move |entry| {
            let key = entry.get();
            let part = list.item(key.clone());
            let state = part.state();
            let shown = memo(move || label(source, &key));
            with_recipe_style(
                node().part(part),
                |r: &ComboboxTheme| r.option_recipe(),
                &variants,
                move || state.get(),
            )
            .child(text(shown))
        }
    });
    let rows = scroll(column(rows))
        .with_node(|node| {
            node.part(match input {
                Some(input) => list.content_for(input),
                None => list.content(),
            })
        })
        .name(name);
    let no_results = show(move || list.visible().with(Vec::is_empty), {
        let variants = variants.clone();
        move || {
            with_recipe_style(
                text(empty),
                |r: &ComboboxTheme| r.empty_recipe(),
                &variants,
                PartState::default,
            )
        }
    });
    with_recipe_style(
        column((rows, no_results)),
        |r: &ComboboxTheme| r.surface_recipe(),
        &variants,
        PartState::default,
    )
    .into_view()
}

/// Draw selected values as removable chips while keeping focus in the editor.
fn selected_chips<K: ItemKey>(
    chips: bool,
    selection: Selection<K>,
    known: Value<Vec<ListItem<K>>>,
    disabled: Value<bool>,
    input_ref: kivact_core::view::NodeRef,
    variants: &kivact_core::view::Variants,
) -> Option<View> {
    if chips {
        match selection {
            Selection::Multiple(values) => Some(
                for_each(values.into_value(), |key| key.clone(), {
                    let variants = variants.clone();
                    move |entry| {
                        let key = entry.get();
                        let shown = memo({
                            let key = key.clone();
                            move || label(known, &key)
                        });
                        let remove = with_recipe_style(
                            button(move || format!("Remove {}", shown.get()))
                                .variant(ButtonVariant::Ghost)
                                .hide_label(true)
                                .icon(icon("x"))
                                .disabled(disabled)
                                .focusable(false)
                                .with_node(move |node| node.focus_on_press(input_ref))
                                .on_press(move || {
                                    values.set(
                                        values
                                            .get_untracked()
                                            .into_iter()
                                            .filter(|item| item != &key)
                                            .collect(),
                                    )
                                }),
                            |r: &ComboboxTheme| r.remove_recipe(),
                            &variants,
                            PartState::default,
                        );
                        with_recipe_style(
                            row((text(shown), remove)),
                            |r: &ComboboxTheme| r.chip_recipe(),
                            &variants,
                            PartState::default,
                        )
                    }
                })
                .into_view(),
            ),
            Selection::Single(_) => None,
        }
    } else {
        None
    }
}

/// Wrap caller writes in one edit and announce only accepted multiple choices.
fn editable_selection<K: ItemKey>(
    selection: Selection<K>,
    retained_labels: Value<Vec<ListItem<K>>>,
    notices: Notices,
    edit: kivact_interaction::EditSession<()>,
) -> Selection<K> {
    match selection {
        Selection::Single(value) => Selection::Single(binding(value, move |next| {
            edit.once((), || value.set(next))
        })),
        Selection::Multiple(value) => Selection::Multiple(binding(value, move |next| {
            edit.once((), || {
                let before = value.get_untracked();
                let labels = retained_labels.get_untracked();
                value.set(next);
                notices.accepted(&before, &value.get_untracked(), &labels);
            })
        })),
    }
}

/// Actual input handles and semantic projections shared by the combobox editor.
struct EditorState<K: ItemKey> {
    list: ComboboxApi<K>,
    selection: Selection<K>,
    known: Value<Vec<ListItem<K>>>,
    name: Value<String>,
    disabled: Value<bool>,
    invalid: Value<bool>,
}
/// Render an accessible query editor with the actual list input authority.
fn editor_view<K: ItemKey>(
    state: EditorState<K>,
    input_ref: kivact_core::view::NodeRef,
    free_text: bool,
    placeholder: Value<String>,
    presentation: crate::SelectPresentation,
    variants: &kivact_core::view::Variants,
    root: &mut kivact_core::view::Node,
) -> View {
    let EditorState {
        list,
        selection,
        known,
        name,
        disabled,
        invalid,
    } = state;
    let shown = list.is_open();
    let tap = kivact_interaction::use_sheet_tap(move || !disabled.get() && !shown.get());
    let filtering = list.is_filtering();
    let query = list.query();
    let displayed = binding(
        memo(move || {
            if !free_text && !filtering.get() {
                match selection {
                    Selection::Single(value) => value.with(|value| {
                        value
                            .as_ref()
                            .map(|key| label(known, key))
                            .unwrap_or_default()
                    }),
                    Selection::Multiple(_) => query.get(),
                }
            } else {
                query.get()
            }
        }),
        move |text| {
            if !disabled.get_untracked() {
                if !shown.get_untracked() {
                    list.show();
                }
                query.set(text);
            }
        },
    );
    let mut editor = with_recipe_style(
        text_input(displayed)
            .label(name)
            .node_ref(input_ref)
            .disabled(disabled)
            .invalid(invalid)
            .placeholder(placeholder)
            .forward_keys(move || {
                let mut keys = vec![Key::ArrowUp, Key::ArrowDown, Key::Enter, Key::Escape];
                if matches!(selection, Selection::Multiple(_)) && query.get().is_empty() {
                    keys.push(Key::Backspace);
                }
                keys
            })
            .with_node(move |node| node.part(list.input()))
            .with_node(move |node| node.part(tap.target()))
            .on_pointer(editor_pointer(list, disabled, tap, presentation)),
        |r: &ComboboxTheme| r.input_recipe(),
        variants,
        PartState::default,
    );
    for reference in root.accessible_labelled_by() {
        editor = editor.with_node(|node| node.labelled_by(reference));
    }
    for reference in root.accessible_described_by() {
        editor = editor.with_node(|node| node.described_by(reference));
    }
    editor.into_view()
}

/// Open the list from a pointer press, or the sheet from a touch tap.
fn editor_pointer<K: ItemKey>(
    list: ComboboxApi<K>,
    disabled: Value<bool>,
    tap: kivact_interaction::SheetTapApi,
    presentation: crate::SelectPresentation,
) -> impl Fn(kivact_core::view::PointerEvent) -> Handled {
    let shown = list.is_open();
    move |event| {
        tap.pointer(event);
        if !disabled.get_untracked() {
            if presentation.uses_sheet() && tap.take_tap() {
                list.show_sheet();
            } else if matches!(event.phase(), PointerPhase::Down(_))
                && ((event.kind() != PointerKind::Touch && !shown.get_untracked())
                    || (event.kind() == PointerKind::Touch && !presentation.uses_sheet()))
            {
                list.show();
            }
        }
        Handled::No
    }
}

/// Apply matching and exclusion using the actual query-filtering phase.
fn filter_options<K: ItemKey>(
    list: ComboboxApi<K>,
    selection: Selection<K>,
    matcher: Option<super::data::Matcher<K>>,
    exclude_selected: bool,
    asynchronous: bool,
) -> ComboboxApi<K> {
    let filtering = list.is_filtering();
    list.filter(move |query, item| {
        if exclude_selected
            && match selection {
                Selection::Single(value) => value.with(|v| v.as_ref() == Some(item.key())),
                Selection::Multiple(value) => value.with(|v| v.contains(item.key())),
            }
        {
            return false;
        }
        asynchronous
            || !filtering.get()
            || match &matcher {
                Some(matcher) => matcher(query, item),
                None => item
                    .label()
                    .to_lowercase()
                    .contains(&query.trim().to_lowercase()),
            }
    })
}

/// Present suggestions inline or in the anchored popup surface.
fn popup_surface<K: ItemKey>(
    list: ComboboxApi<K>,
    source: Value<Vec<ListItem<K>>>,
    name: Value<String>,
    empty: Value<String>,
    variants: kivact_core::view::Variants,
    anchor: kivact_core::view::NodeRef,
    inline: bool,
) -> View {
    let shown = list.is_open();
    let sheet_shown = list.is_sheet_open();
    let surface_variants = variants.clone();
    show(
        move || shown.get() && !sheet_shown.get(),
        move || {
            let content = suggestions(list, source, name, empty, surface_variants.clone(), None);
            if inline {
                content
            } else {
                overlay(anchor, Placement::Below, content)
                    .at_least_anchor_width()
                    .offset(
                        kivact_core::style::use_theme()
                            .get()
                            .get(&crate::combobox::look::POPUP_OFFSET_VAR),
                    )
                    .on_dismiss(move |_| list.hide())
                    .into_view()
            }
        },
    )
    .into_view()
}

/// Present the shared query and suggestions in a touch-oriented drawer.
fn sheet_surface<K: ItemKey>(
    list: ComboboxApi<K>,
    source: Value<Vec<ListItem<K>>>,
    name: Value<String>,
    empty: Value<String>,
    variants: kivact_core::view::Variants,
    disabled: Value<bool>,
    placeholder: Value<String>,
) -> View {
    let sheet_shown = list.is_sheet_open();
    let query = list.query();
    let sheet_open = binding(sheet_shown, move |shown| {
        if !shown {
            list.hide();
        }
    });
    let sheet_input = node_ref();
    let sheet_search = kivact_core::i18n::message("combobox.sheet_search", "Search options");
    crate::dialog(name, sheet_open)
        .mode(crate::dialog::DialogKind::Drawer(
            kivact_core::geometry::Edge::Bottom,
        ))
        .initial_focus(sheet_input)
        .content(move || {
            column((
                text_input(query)
                    .label(sheet_search)
                    .node_ref(sheet_input)
                    .disabled(disabled)
                    .placeholder(placeholder)
                    .forward_keys(vec![
                        Key::ArrowUp,
                        Key::ArrowDown,
                        Key::Enter,
                        Key::Escape,
                        Key::Backspace,
                    ])
                    .with_node(move |node| node.part(list.input_for(sheet_input))),
                suggestions(
                    list,
                    source,
                    name,
                    empty,
                    variants.clone(),
                    Some(sheet_input),
                ),
            ))
        })
        .into_view()
}

/// Shared handles and read-only inputs prepared before view composition.
struct Mounted<K: ItemKey> {
    list: ComboboxApi<K>,
    selection: Selection<K>,
    known: Value<Vec<ListItem<K>>>,
    notices: Notices,
    name: Value<String>,
    disabled: Value<bool>,
    invalid: Value<bool>,
    inline: bool,
}
/// Configure shared interaction and retain the labels required by its selection.
fn prepare<K: ItemKey>(field: &mut Combobox<K>) -> Mounted<K> {
    let selection = field.selection;
    let named = is_named(&mut field.root) || field.label.is_some();
    report_unnamed(named, "combobox", field.at);
    let name = field
        .root
        .accessible_name()
        .or(field.label)
        .unwrap_or_else(|| String::new().into_value());
    let disabled = field
        .root
        .accessible_disabled()
        .unwrap_or_else(|| false.into_value());
    let invalid = field
        .root
        .accessible_invalid()
        .unwrap_or_else(|| false.into_value());
    let edit = kivact_interaction::EditSession::new(|(): ()| {});
    if let Some(observer) = field.on_edit {
        edit.on_edit(move |phase| observer.call(phase));
    }
    let known = known_options(field.options);
    let retained_labels = known.values();
    let notices = Notices::new();
    let selection = editable_selection(selection, retained_labels, notices, edit);
    let asynchronous = field.search.is_some();
    let (mut list, inline) = configured_list(
        field.options,
        selection,
        field.query,
        field.open,
        field.mode,
        field.search.clone(),
        field.debounce,
    );
    kivact_core::state::effect_on(disabled, move |disabled| {
        if *disabled {
            list.hide();
        }
    });
    let source = list.options();
    let options = field.options;
    let label_source = memo(move || {
        let mut rows = source.get();
        for item in options.get() {
            if !rows.iter().any(|row| row.key() == item.key()) {
                rows.push(item);
            }
        }
        rows
    });
    known.follow(label_source, selection);
    let known = known.values();
    list = filter_options(
        list,
        selection,
        field.matcher.clone(),
        field.exclude_selected,
        asynchronous,
    );
    Mounted {
        list,
        selection,
        known,
        notices,
        name,
        disabled,
        invalid,
        inline,
    }
}

/// Configure popup bindings and asynchronous search before applying filters.
fn configured_list<K: ItemKey>(
    options: Value<Vec<ListItem<K>>>,
    selection: Selection<K>,
    controlled: Option<kivact_core::state::Signal<String>>,
    controlled_open: Option<kivact_core::state::Signal<bool>>,
    mode: Option<ComboboxMode>,
    search: Option<super::data::Search<K>>,
    debounce: Value<std::time::Duration>,
) -> (ComboboxApi<K>, bool) {
    let inline = matches!(mode.unwrap_or(ComboboxMode::Popup), ComboboxMode::Inline);
    let query = controlled.unwrap_or_else(|| signal(String::new()));
    let open = if inline {
        signal(true)
    } else {
        controlled_open.unwrap_or_else(|| signal(false))
    };
    let mut list = use_combobox(options, selection, query, open, ComboboxOptions { inline })
        .debounce(debounce);
    if let Some(search) = search {
        list = list.search(move |query| search(query));
    }
    (list, inline)
}

/// Render clear and popup actions without transferring focus from the editor.
fn editor_actions<K: ItemKey>(
    list: ComboboxApi<K>,
    selection: Selection<K>,
    clear: Value<bool>,
    disabled: Value<bool>,
    input_ref: kivact_core::view::NodeRef,
    inline: bool,
) -> (Option<View>, Option<View>) {
    let query = list.query();
    let clear_button = match selection {
        Selection::Single(value) => Some(
            show(
                move || clear.get() && value.with(Option::is_some),
                move || {
                    button(kivact_core::i18n::message("combobox.clear", "Clear"))
                        .variant(ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(icon("x"))
                        .disabled(disabled)
                        .focusable(false)
                        .with_node(move |node| node.focus_on_press(input_ref))
                        .on_press(move || {
                            value.set(None);
                            query.set(String::new());
                        })
                },
            )
            .into_view(),
        ),
        Selection::Multiple(_) => None,
    };
    let toggle = (!inline).then(|| {
        button(kivact_core::i18n::message(
            "combobox.toggle",
            "Toggle options",
        ))
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .icon(icon("chevron-down"))
        .disabled(disabled)
        .focusable(false)
        .with_node(move |node| node.focus_on_press(input_ref))
        .on_press(move || list.toggle())
        .into_view()
    });
    (clear_button, toggle)
}
