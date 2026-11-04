//! Role: Render a themed select using the shared keyed selection parts.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::values::{ControlSize, SelectTriggerState};
use super::variants::{Variant, Variants};
use kivact_core::custom::NodeExt;
use kivact_core::geometry::Point;
use kivact_core::state::{Binding, Callback, Value, binding, effect_on, signal, var};
use kivact_core::state::{IntoText, IntoValue, callback};
use kivact_core::style::style;
use kivact_core::view::HandlerResult;
use kivact_core::view::{
    Handled, IntoView, NodeRef, Placement, PointerId, PointerKind, PointerPhase, PopupKind, View,
    for_each, overlay, show,
};
use kivact_core::view::{Node, NodeBuilder, column, node, scroll, text};
use kivact_interaction::{ListItem, ListItemState, PopupState};
use std::hash::Hash;
use std::rc::Rc;

/// What an option row shows, from its item and its state.
type OptionContent<K> = Rc<dyn Fn(Value<ListItem<K>>, Value<ListItemState>) -> View>;

/// Recipe variant supplied to options mounted in a select's touch sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectSheetOption(pub ControlSize);

impl Variant for SelectSheetOption {}

type SheetContent = Rc<dyn Fn() -> Node>;
type SheetFactory = Rc<dyn Fn(Value<String>, Binding<bool>, NodeRef, SheetContent) -> View>;

pub struct Select<K: Clone + Eq + Hash + 'static> {
    root: Node,
    label: Value<String>,
    options: Value<Vec<ListItem<K>>>,
    selected: Binding<Option<K>>,
    placeholder: Option<Value<String>>,
    preview: Option<Callback<Option<K>>>,
    icon: Option<View>,
    size: ControlSize,
    option: Option<OptionContent<K>>,
    variants: Variants,
    arrow_open: bool,
    retain_trigger_focus: bool,
    touch_sheet: Option<(Rc<dyn Fn() -> bool>, SheetFactory)>,
}
/// A themed keyed select whose value, focus and keys belong to interaction.
///
/// The trigger shows the selected option's label and the theme's
/// `chevron-down` icon; an option row shows its icon, its label over its
/// description and, while it is the selected one, the theme's `check` icon.
/// [`option`](Select::option) replaces what an option row shows, the check
/// included.
pub fn select<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    selected: impl Into<Binding<Option<K>>>,
) -> Select<K> {
    let label = label.into_text_value();
    let variants = Variants::default();
    Select {
        root: node().name(label),
        label,
        options: options.into_value(),
        selected: selected.into(),
        placeholder: None,
        preview: None,
        icon: None,
        size: super::values::default_control_size(),
        option: None,
        variants,
        arrow_open: false,
        retain_trigger_focus: false,
        touch_sheet: None,
    }
}
impl<K: Clone + Eq + Hash + 'static> Select<K> {
    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which every recipe of the component reads with
    /// [`Look::variant`](crate::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipes read the values given before
    /// the component is mounted, and a value never changes afterwards. A
    /// look that changes while the component lives is a state the recipe
    /// reads, or a reactive `.style(..)` of the caller's.
    pub fn variant(self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }
}
impl<K: Clone + Eq + Hash + 'static> Select<K> {
    /// Open the options on a bare ArrowDown or ArrowUp without selecting.
    pub fn open_on_arrow(mut self, enabled: bool) -> Self {
        self.arrow_open = enabled;
        self
    }

    /// Keep keyboard focus on the trigger while highlighting options.
    pub fn retain_trigger_focus(mut self, enabled: bool) -> Self {
        self.retain_trigger_focus = enabled;
        self
    }

    /// Present touch taps in an alternate modal surface when eligible.
    /// Mouse and keyboard always use the regular popup. The modal factory
    /// receives the label, its open binding, trigger ref, and listbox builder.
    pub fn touch_sheet<V: IntoView>(
        mut self,
        eligible: impl Fn() -> bool + 'static,
        factory: impl Fn(Value<String>, Binding<bool>, NodeRef, Rc<dyn Fn() -> Node>) -> V + 'static,
    ) -> Self {
        self.touch_sheet = Some((
            Rc::new(eligible),
            Rc::new(move |label, open, anchor, content| {
                factory(label, open, anchor, content).into_view()
            }),
        ));
        self
    }

    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.placeholder = Some(text.into_text_value());
        self
    }
    pub fn on_preview<R: HandlerResult>(
        mut self,
        preview: impl Fn(Option<K>) -> R + 'static,
    ) -> Self {
        self.preview = Some(callback(preview));
        self
    }
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    /// The size (default [`ControlSize::Medium`]), given to the trigger
    /// recipe in [`SelectTriggerState::size`]; the recipe sizes the trigger
    /// by it.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// What each option row shows, built once per row from its item and
    /// its state (both reactive). The row itself, its keys, its semantics
    /// and its recipe stay the component's.
    ///
    /// ```no_run
    /// use kivact::prelude::*;
    ///
    /// fn fruit() -> impl IntoView {
    ///     let chosen = signal(None::<&str>);
    ///     let options = vec![ListItem::new("apple", "Apple"), ListItem::new("pear", "Pear")];
    ///     select("Fruit", options, chosen).option(|item, state| {
    ///         text(move || {
    ///             let mark = if state.get().selected { "* " } else { "" };
    ///             format!("{mark}{}", item.get().label())
    ///         })
    ///     })
    /// }
    /// ```
    pub fn option<V: IntoView>(
        mut self,
        content: impl Fn(Value<ListItem<K>>, Value<ListItemState>) -> V + 'static,
    ) -> Self {
        self.option = Some(Rc::new(move |item, state| content(item, state).into_view()));
        self
    }
}
impl<K: Clone + Eq + Hash + 'static> NodeBuilder for Select<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl<K: Clone + Eq + Hash + 'static> IntoView for Select<K> {
    fn into_view(mut self) -> View {
        let mut behavior = kivact_interaction::use_select(self.options, self.selected)
            .open_on_arrow(self.arrow_open)
            .retain_trigger_focus(self.retain_trigger_focus);
        if let Some(text) = self.placeholder {
            behavior = behavior.placeholder(text);
        }
        if let Some(preview) = self.preview {
            behavior = behavior.on_preview(move |key| preview.try_call(key));
        }
        let trigger = behavior.trigger();
        let state = trigger.state();
        let (options, selected, size) = (self.options, self.selected.into_value(), self.size);
        let trigger_state = kivact_core::memo(move || {
            let PopupState { input, open } = state.get();
            let placeholder = selected.with(|key| {
                key.as_ref()
                    .is_none_or(|key| options.with(|rows| rows.iter().all(|row| row.key() != key)))
            });
            SelectTriggerState {
                input,
                open,
                placeholder,
                size,
            }
        });
        self.root = with_base_recipe_style(
            self.root,
            |r| r.select_recipes().trigger_recipe(),
            &self.variants,
            move || trigger_state.get(),
        );
        let anchor = trigger.node_ref();
        let name = self.root.accessible_name();
        let fallback = self.label;
        let label = kivact_core::memo(move || {
            name.as_ref()
                .map(|read| read())
                .unwrap_or_else(|| fallback.get())
        });
        let variants = self.variants;
        let indicator = with_recipe_style(
            node(),
            |r| r.select_recipes().indicator_recipe(),
            &variants,
            move || state.get(),
        )
        .child(kivact_core::view::icon("chevron-down"));
        let shown = behavior.shown();
        let open = behavior.open_binding();
        let sheet_open = signal(false);
        let (trigger, sheet_factory) = if let Some((eligible, factory)) = self.touch_sheet {
            let disabled = self.root.accessible_disabled();
            let contact = var(None::<(PointerId, Point, bool)>);
            // Native activation is a separate event after pointer release.
            // Consume the decision there so a touch never toggles twice.
            let released = var(None::<bool>);
            let bounds = anchor.rect();
            effect_on(shown, move |shown| {
                if !shown {
                    sheet_open.set(false);
                }
            });
            let trigger = trigger
                .on_pointer(move |event| {
                    if matches!(event.phase(), PointerPhase::Down(_)) {
                        released.set(None);
                        if event.kind() == PointerKind::Touch
                            && event.is_primary()
                            && !shown.get()
                            && !disabled.as_ref().is_some_and(|read| read())
                        {
                            contact.set(Some((event.id(), event.window_pos(), false)));
                        }
                        return Handled::No;
                    }
                    let Some((id, start, moved)) =
                        contact.get().filter(|(id, _, _)| *id == event.id())
                    else {
                        return Handled::No;
                    };
                    let pos = event.window_pos();
                    let moved = moved || (pos.x - start.x).hypot(pos.y - start.y) > 10.0;
                    match event.phase() {
                        PointerPhase::Move => contact.set(Some((id, start, moved))),
                        PointerPhase::Up(_) => {
                            contact.set(None);
                            released.set(
                                bounds
                                    .get()
                                    .is_some_and(|rect| rect.contains(pos))
                                    .then_some(!moved),
                            );
                        }
                        PointerPhase::Cancel => {
                            contact.set(None);
                            released.set(None);
                        }
                        _ => {}
                    }
                    Handled::No
                })
                .on_key(move |_| {
                    released.set(None);
                    Handled::No
                })
                .on_press(move || {
                    let choice = released.get();
                    released.set(None);
                    match choice {
                        Some(true) if eligible() => {
                            sheet_open.set(true);
                            open.set(true);
                        }
                        Some(false) => {}
                        _ => behavior.toggle(),
                    }
                })
                .has_popup(move || {
                    Some(if sheet_open.get() {
                        PopupKind::Dialog
                    } else {
                        PopupKind::ListBox
                    })
                });
            (trigger, Some(factory))
        } else {
            (trigger, None)
        };
        let trigger = self
            .root
            .part(trigger)
            .child(self.icon)
            .child(text(behavior.label()).style(style().grow(1).single_line()))
            .child(indicator);
        let options = self.options;
        let slot = self.option;
        let body_variants = variants.clone();
        let build = Rc::new(move |sheet: bool| {
            let slot = slot.clone();
            let rows = if sheet {
                body_variants.with(SelectSheetOption(size))
            } else {
                body_variants.clone()
            };
            for_each(
                options,
                |item| item.key().clone(),
                move |key, item| {
                    let variants = &rows;
                    let lines = rows.clone();
                    let part = if sheet {
                        behavior.sheet_item(key)
                    } else {
                        behavior.item(key)
                    };
                    let state = part.state();
                    let content = match &slot {
                        Some(slot) => slot(item, state),
                        None => (
                            super::helpers::item_icon(move || item.get().icon_value().cloned()),
                            super::helpers::label_and_description(
                                move || item.get().label(),
                                move || item.get().description_value(),
                                move |line| {
                                    with_recipe_style(
                                        line,
                                        |r| r.select_recipes().description_recipe(),
                                        &lines,
                                        move || state.get(),
                                    )
                                },
                            ),
                            with_recipe_style(
                                node(),
                                |r| r.select_recipes().check_recipe(),
                                variants,
                                move || state.get(),
                            )
                            .child(kivact_core::view::dynamic(
                                move || state.get().selected,
                                |selected| selected.then(|| kivact_core::view::icon("check")),
                            )),
                        )
                            .into_view(),
                    };
                    with_recipe_style(
                        node().part(part),
                        |r| r.select_recipes().option_recipe(),
                        variants,
                        move || state.get(),
                    )
                    .child(content)
                },
            )
        });
        let popup_build = build.clone();
        let surface = show(
            move || shown.get() && !sheet_open.get(),
            move || {
                let part = behavior.content();
                let state = part.state();
                overlay(
                    anchor,
                    Placement::Below,
                    with_recipe_style(
                        scroll(column(popup_build(false))),
                        |r| r.select_recipes().surface_recipe(),
                        &variants,
                        move || state.get(),
                    ),
                )
                .part(part)
                .name(label)
                .offset(4.0)
                .at_least_anchor_width(true)
            },
        );
        let sheet = sheet_factory.map(|factory| {
            let visible = binding(
                move || shown.get() && sheet_open.get(),
                move |show| {
                    if !show {
                        open.set(false);
                    }
                },
            );
            factory(
                label,
                visible,
                anchor,
                Rc::new(move || {
                    column(build(true))
                        .part(behavior.sheet_content())
                        .name(label)
                }),
            )
        });
        (trigger, surface, sheet).into_view()
    }
}
