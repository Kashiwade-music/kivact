//! Role: Searchable keyed suggestions with a native editor and an anchored list.
use crate::recipes_field::{frame, text as text_style};
use crate::theme_access::tokens;
use kivact::control::{Node, NodeBuilder, column, node, row, scroll, text};
use kivact::i18n::message;
use kivact::interaction::{ListSelection, use_combobox};
use kivact::primitive::{NodeExt, PartState, part};
use kivact::style::{Align, Color, pct, style};
use kivact::task::{AsyncState, resource, sleep};
use kivact::ui::{ListItem, text_input};
use kivact::view::{
    Handled, IntoText, IntoValue, IntoView, Key, OverlayAlign, Placement, PointerEvent,
    PointerPhase, View, for_each, node_ref, overlay, show,
};
use kivact::{Binding, Result, Value, binding, effect, memo, signal};
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

type Search<K> = Rc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<Vec<ListItem<K>>>>>>>;

pub struct Typeahead<K: Clone + Eq + Hash + 'static> {
    pub(super) root: Node,
    pub(super) label: Value<String>,
    pub(super) options: Value<Vec<ListItem<K>>>,
    pub(super) selection: ListSelection<K>,
    pub(super) chip_labels: Option<Value<Vec<ListItem<K>>>>,
    pub(super) chips: Option<Binding<Vec<K>>>,
    placeholder: Value<String>,
    empty: Value<String>,
    search: Option<Search<K>>,
    debounce: Duration,
    exclude_selected: bool,
}

pub fn typeahead<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    selected: impl Into<Binding<Option<K>>>,
) -> Typeahead<K> {
    Typeahead::new(label, options, ListSelection::Single(selected.into()))
}

pub fn base_typeahead<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    selected: impl Into<Binding<Option<K>>>,
) -> Typeahead<K> {
    typeahead(label, options, selected)
}

impl<K: Clone + Eq + Hash + 'static> Typeahead<K> {
    pub(super) fn new(
        label: impl IntoText,
        options: impl IntoValue<Vec<ListItem<K>>>,
        selection: ListSelection<K>,
    ) -> Self {
        Self {
            root: node(),
            label: label.into_text_value(),
            options: options.into_value(),
            selection,
            chip_labels: None,
            chips: None,
            placeholder: message("soft.typeahead.placeholder", "Search"),
            empty: message("soft.typeahead.empty", "No results found."),
            search: None,
            debounce: Duration::from_millis(150),
            exclude_selected: false,
        }
    }

    /// Fetch suggestions for the current query. The source runs immediately when
    /// the list opens and after the debounce interval when the user types.
    /// Closing the list or changing the query drops the previous future.
    pub fn search<F, Fut>(mut self, search: F) -> Self
    where
        F: Fn(String) -> Fut + 'static,
        Fut: Future<Output = Result<Vec<ListItem<K>>>> + 'static,
    {
        self.search = Some(Rc::new(move |query| Box::pin(search(query))));
        self
    }

    /// Delay between the last keystroke and starting an asynchronous search.
    pub fn debounce(mut self, delay: Duration) -> Self {
        self.debounce = delay;
        self
    }

    pub(super) fn exclude_selected(mut self) -> Self {
        self.exclude_selected = true;
        self
    }

    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.placeholder = text.into_text_value();
        self
    }

    pub fn empty_text(mut self, text: impl IntoText) -> Self {
        self.empty = text.into_text_value();
        self
    }
}

impl<K: Clone + Eq + Hash + 'static> NodeBuilder for Typeahead<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

fn label_of<K: Clone + Eq + Hash + 'static>(options: Value<Vec<ListItem<K>>>, key: &K) -> String {
    options.with(|items| {
        items
            .iter()
            .find(|item| item.key() == key)
            .map(|item| item.label())
            .unwrap_or_default()
    })
}

impl<K: Clone + Eq + Hash + 'static> IntoView for Typeahead<K> {
    fn into_view(self) -> View {
        let Self {
            mut root,
            label,
            options,
            selection,
            chips,
            chip_labels,
            placeholder,
            empty,
            search,
            debounce,
            exclude_selected,
        } = self;
        let off = root.accessible_disabled();
        let disabled = memo(move || off.as_ref().is_some_and(|off| off()));
        let single = match selection {
            ListSelection::Single(value) if chips.is_none() => Some(value),
            _ => None,
        };
        let open = signal(false);
        let editing = signal(false);
        let typed = signal(String::new());
        let asynchronous = search.map(|search| {
            let request = memo(move || {
                (open.get() && !disabled.get()).then(|| {
                    if editing.get() {
                        (typed.get(), debounce)
                    } else {
                        (String::new(), Duration::ZERO)
                    }
                })
            });
            resource(request, move |request| {
                let search = search.clone();
                async move {
                    let Some((query, delay)) = request else {
                        return Ok(None);
                    };
                    if !delay.is_zero() {
                        sleep(delay).await;
                    }
                    search(query).await.map(Some)
                }
            })
        });
        let cached = signal(Vec::<ListItem<K>>::new());
        if let Some(search) = asynchronous {
            effect(move || {
                if let Some(Some(results)) = search.latest() {
                    let mut retained = results;
                    let one = single.and_then(|value| value.get());
                    let many = chips.map(|values| values.get()).unwrap_or_default();
                    for item in cached.get_untracked() {
                        if (one.as_ref() == Some(item.key()) || many.contains(item.key()))
                            && !retained.iter().any(|result| result.key() == item.key())
                        {
                            retained.push(item);
                        }
                    }
                    cached.set(retained);
                }
            });
        }
        let labels = chip_labels.unwrap_or(options);
        let labels = memo(move || {
            let mut known = cached.get();
            for item in labels.get() {
                if !known.iter().any(|result| result.key() == item.key()) {
                    known.push(item);
                }
            }
            known
        });
        let options = asynchronous.map_or(options, |search| {
            memo(move || {
                let AsyncState::Ready(Some(mut results)) = search.state() else {
                    return Vec::new();
                };
                if exclude_selected && let Some(chips) = chips {
                    chips.with(|chosen| results.retain(|item| !chosen.contains(item.key())));
                }
                results
            })
        });
        let list = use_combobox(options, selection)
            .popup(open)
            .filter(move |query, item| {
                asynchronous.is_some()
                    || !editing.get()
                    || item
                        .label()
                        .to_lowercase()
                        .contains(&query.trim().to_lowercase())
            });
        let query = list.query();
        effect(move || {
            if !open.get() {
                editing.set(false);
                query.set(
                    single
                        .and_then(|value| value.get())
                        .map(|key| label_of(labels, &key))
                        .unwrap_or_default(),
                );
            }
        });
        effect(move || {
            if disabled.get() {
                open.set(false);
            }
        });
        let edit = binding(query, move |text: String| {
            editing.set(true);
            typed.set(text.clone());
            query.set(text);
            open.set(true);
        });
        let input_part = list.input();
        let input_ref = input_part.node_ref();
        let input = text_input(label, edit)
            .placeholder(placeholder)
            .disabled(disabled)
            .forward_keys(vec![Key::ArrowUp, Key::ArrowDown, Key::Enter, Key::Escape])
            .part(input_part)
            .expanded(open)
            .on_pointer(move |event: PointerEvent| {
                if matches!(event.phase, PointerPhase::Down(_)) && !disabled.get() {
                    open.set(true);
                }
                Handled::No
            })
            .style(move || {
                style()
                    .grow(1)
                    .min_width(64)
                    .height(24)
                    .padding(0)
                    .border(0, Color::TRANSPARENT)
                    .ring_none()
                    .background(Color::TRANSPARENT)
                    .hover(style().ring_none())
                    .focus(style().ring_none())
            });
        let anchor = node_ref();
        let focused = anchor.focus_within();
        let chip_labels = labels;
        let tokens_view = chips.map(|values| {
            for_each(
                values.into_value(),
                |key| key.clone(),
                move |key, _| {
                    let shown = memo({
                        let key = key.clone();
                        move || label_of(chip_labels, &key)
                    });
                    let remove = kivact::ui::button(move || format!("Remove {}", shown.get()))
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::control::icon("x"))
                        .disabled(disabled)
                        .part(part(|s| s).focus_on_press(input_ref))
                        .on_press(move || {
                            values.set(
                                values
                                    .get_untracked()
                                    .into_iter()
                                    .filter(|item| item != &key)
                                    .collect(),
                            )
                        })
                        .style(style().size(16, 16).padding(0));
                    row((text(shown), remove)).style(move || {
                        let t = tokens();
                        style()
                            .height(24)
                            .padding_left(8)
                            .padding_right(4)
                            .gap(4)
                            .align_items(Align::Center)
                            .radius(t.radius.element)
                            .background(t.colors.accent_muted)
                            .color(t.colors.text_primary)
                    })
                },
            )
            .into_view()
        });
        let field = row((tokens_view, input))
            .node_ref(anchor)
            .part(part(|s| s).focus_on_press(input_ref))
            .style(move || {
                let t = tokens();
                let s = frame(
                    &t,
                    &PartState {
                        focused: focused.get(),
                        ..PartState::default()
                    },
                    false,
                )
                .min_height(32)
                .padding_x(8)
                .padding_y(3)
                .gap(4)
                .align_items(Align::Center)
                .wrap(chips.is_some());
                if focused.get() {
                    s.border_color(t.colors.accent)
                        .shadow(t.shadows.input_focus_ring)
                } else {
                    s
                }
            });
        let popup = show(open, move || {
            let rows = for_each(
                list.visible(),
                |key| key.clone(),
                move |key, _| {
                    let item = list.item(key.clone());
                    let state = item.state();
                    let text_value = memo({
                        let key = key.clone();
                        move || label_of(options, &key)
                    });
                    node().part(item).child(text(text_value)).style(move || {
                        let t = tokens();
                        let s = text_style(
                            style()
                                .min_height(32)
                                .padding_x(8)
                                .padding_y(6)
                                .radius(t.radius.element),
                            t.typography.body,
                            t.colors.text_primary,
                        );
                        if state.get().active {
                            s.background(t.colors.overlay_hover)
                        } else {
                            s
                        }
                    })
                },
            );
            let no_results = memo(move || list.visible().with(Vec::is_empty));
            let loading = message("soft.typeahead.loading", "Loading…");
            let failed = message("soft.typeahead.failed", "Could not load suggestions.");
            let status = move || match asynchronous.map(|search| search.state()) {
                Some(AsyncState::Pending) => loading.get(),
                Some(AsyncState::Failed(_)) => failed.get(),
                _ => empty.get(),
            };
            let surface = column((
                scroll(column(rows))
                    .part(list.content())
                    .name(label)
                    .style(style().max_height(292)),
                show(no_results, move || {
                    text(status)
                        .style(move || style().padding(8).color(tokens().colors.text_secondary))
                }),
            ))
            .style(move || {
                let t = tokens();
                t.shadows.low.iter().copied().fold(
                    style()
                        .width(pct(100))
                        .max_height(300)
                        .padding(4)
                        .radius(t.radius.container)
                        .background(t.colors.bg_popover),
                    |s, shadow| s.shadow(shadow),
                )
            });
            overlay(anchor, Placement::Below, surface)
                .align(OverlayAlign::Start)
                .offset(4)
                .at_least_anchor_width(true)
                .on_dismiss(move |_| open.set(false))
        });
        root.child(field).child(popup).into_view()
    }
}
