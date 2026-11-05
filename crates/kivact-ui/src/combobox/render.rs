//! Role: Render one combobox query and selection through themed editor and list parts.
use super::data::Combobox;
use super::labels::{KnownLabels, label};
use super::notices::Notices;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::{ButtonVariant, button, text_input};
use kivact_core::custom::PartState;
use kivact_core::state::{IntoBinding, IntoValue, ItemKey, Value, binding, memo, signal};
use kivact_core::view::{
    Handled, IntoView, Key, NodeBuilder, Placement, PointerKind, PointerPhase, View, column,
    for_each, icon, node, node_ref, overlay, row, scroll, show, text,
};
use kivact_interaction::{ComboboxApi, ComboboxMode, ListItem, Selection, use_combobox};

pub(super) fn render<K: ItemKey>(field: Combobox<K>) -> View {
    let Combobox {
        mut root,
        label: name,
        options,
        selection,
        query: controlled,
        open: controlled_open,
        on_edit,
        free_text,
        mode,
        chips,
        placeholder,
        empty,
        clear,
        matcher,
        search,
        debounce,
        exclude_selected,
        presentation,
        variants,
    } = field;
    let name = root.accessible_name().unwrap_or(name);
    let disabled = root
        .accessible_disabled()
        .unwrap_or_else(|| false.into_value());
    let invalid = root
        .accessible_invalid()
        .unwrap_or_else(|| false.into_value());
    let edit = kivact_interaction::EditSession::new(|(): ()| {});
    if let Some(observer) = on_edit {
        edit.on_edit(move |phase| observer.call(phase));
    }
    let known = KnownLabels::new(options);
    let retained_labels = known.values();
    let notices = Notices::new();
    let selection = match selection {
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
    };
    let local_open = signal(false);
    let (inline, open) = match mode.unwrap_or(ComboboxMode::Popup) {
        ComboboxMode::Inline => (true, binding(|| true, |_: bool| {})),
        ComboboxMode::Popup => (
            false,
            controlled_open.unwrap_or_else(|| local_open.into_binding()),
        ),
    };
    let mut list = use_combobox(options, selection).debounce(debounce);
    if let Some(query) = controlled {
        list = list.query_binding(query);
    }
    if !inline {
        list = list.popup_with_focus_transfer(open, list.is_sheet_open());
    }
    let asynchronous = search.is_some();
    if let Some(search) = search {
        list = list.search(move |query| search(query));
    }
    kivact_core::state::effect_on(disabled, move |disabled| {
        if *disabled {
            list.close();
        }
    });
    let shown = list.is_open();
    let sheet_shown = list.is_sheet_open();
    let source = list.options();
    let label_source = memo(move || {
        let mut rows = source.get();
        for item in options.get() {
            if !rows.iter().any(|row| row.key() == item.key()) {
                rows.push(item);
            }
        }
        rows
    });
    known.observe(label_source, selection);
    let known = known.values();
    let filtering = list.is_filtering();
    list = list.filter(move |query, item| {
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
    });
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
                    list.open();
                }
                query.set(text);
            }
        },
    );
    let input_ref = node_ref();
    let anchor = node_ref();
    let within = anchor.focus_within();
    let bounds = input_ref.rect();
    let gesture = kivact_core::state::machine(
        crate::select::machine::SheetGesture::Idle,
        move || crate::select::machine::Machine {
            readiness: if !disabled.get() && !shown.get() {
                crate::select::machine::Readiness::Ready
            } else {
                crate::select::machine::Readiness::Unavailable
            },
            bounds: bounds.get(),
            slop: kivact_core::style::use_theme()
                .get()
                .ext::<kivact_interaction::InteractionPolicy>()
                .select
                .sheet_slop,
        },
        |_, effect| match effect {},
    );
    let placeholder =
        placeholder.unwrap_or_else(|| kivact_core::i18n::message("combobox.placeholder", "Search"));
    let mut editor = with_recipe_style(
        text_input(name, displayed)
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
            .on_pointer(move |event| {
                gesture.send(crate::select::machine::Event::Pointer(event));
                if !disabled.get_untracked() {
                    if presentation.uses_sheet()
                        && matches!(
                            gesture.get(),
                            crate::select::machine::SheetGesture::Released(
                                crate::select::machine::Outcome::Tap
                            )
                        )
                    {
                        list.open_sheet();
                        gesture.send(crate::select::machine::Event::Reset);
                    } else if event.kind() != PointerKind::Touch
                        && matches!(event.phase(), PointerPhase::Down(_))
                        && !shown.get_untracked()
                    {
                        list.open();
                    } else if event.kind() == PointerKind::Touch
                        && !presentation.uses_sheet()
                        && matches!(event.phase(), PointerPhase::Down(_))
                    {
                        list.open();
                    }
                }
                Handled::No
            }),
        |r| r.combobox_recipes().input_recipe(),
        &variants,
        PartState::default,
    );
    for reference in root.accessible_labelled_by() {
        editor = editor.with_node(|node| node.labelled_by(reference));
    }
    for reference in root.accessible_described_by() {
        editor = editor.with_node(|node| node.described_by(reference));
    }
    let tokens = if chips {
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
                            |r| r.combobox_recipes().remove_recipe(),
                            &variants,
                            PartState::default,
                        );
                        with_recipe_style(
                            row((text(shown), remove)),
                            |r| r.combobox_recipes().chip_recipe(),
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
    };
    let clear_button = match selection {
        Selection::Single(value) => Some(
            show(
                move || clear.get() && value.with(Option::is_some),
                move || {
                    button("Clear")
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
        button("Toggle options")
            .variant(ButtonVariant::Ghost)
            .hide_label(true)
            .icon(icon("chevron-down"))
            .disabled(disabled)
            .focusable(false)
            .with_node(move |node| node.focus_on_press(input_ref))
            .on_press(move || list.toggle())
    });
    let frame = with_base_recipe_style(
        row((editor, tokens, clear_button, toggle))
            .node_ref(anchor)
            .with_node(move |node| node.focus_on_press(input_ref)),
        |r| r.combobox_recipes().frame_recipe(),
        &variants,
        move || PartState {
            focused: within.get(),
            disabled: disabled.get(),
            ..PartState::default()
        },
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
    let surface_variants = variants.clone();
    let surface = show(
        move || shown.get() && !sheet_shown.get(),
        move || {
            let content = suggestions(list, source, name, empty, surface_variants.clone());
            if inline {
                content
            } else {
                overlay(anchor, Placement::Below, content)
                    .at_least_anchor_width()
                    .offset(
                        kivact_core::style::use_theme()
                            .get()
                            .ext::<crate::UiTokens>()
                            .combobox
                            .popup_offset,
                    )
                    .on_dismiss(move |_| list.close())
                    .into_view()
            }
        },
    );
    let sheet_open = binding(sheet_shown, move |shown| {
        if !shown {
            list.close();
        }
    });
    let sheet_input = node_ref();
    let sheet_list = list.with_input_ref(sheet_input);
    let sheet_search = kivact_core::i18n::message("combobox.sheet_search", "Search options");
    let sheet = crate::dialog(name, sheet_open)
        .mode(kivact_interaction::DialogMode::Drawer(
            kivact_core::geometry::Edge::Bottom,
        ))
        .initial_focus(sheet_input)
        .content(move || {
            column((
                text_input(sheet_search, query)
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
                    .with_node(move |node| node.part(sheet_list.input())),
                suggestions(sheet_list, source, name, empty, variants.clone()),
            ))
        });
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
    variants: crate::variants::Variants,
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
                |r| r.combobox_recipes().option_recipe(),
                &variants,
                move || state.get(),
            )
            .child(text(shown))
        }
    });
    let rows = scroll(column(rows))
        .with_node(|node| node.part(list.content()))
        .name(name);
    let no_results = show(move || list.visible().with(Vec::is_empty), {
        let variants = variants.clone();
        move || {
            with_recipe_style(
                text(empty),
                |r| r.combobox_recipes().empty_recipe(),
                &variants,
                PartState::default,
            )
        }
    });
    with_recipe_style(
        column((rows, no_results)),
        |r| r.combobox_recipes().surface_recipe(),
        &variants,
        PartState::default,
    )
    .into_view()
}
