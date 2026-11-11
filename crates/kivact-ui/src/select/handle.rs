//! Role: Render a themed select using the shared keyed selection parts.
use super::machine;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use crate::values::{ControlSize, SelectTriggerState};
use crate::variants::{Variant, Variants};
use kivact_core::state::{Binding, Callback, Value, binding};
use kivact_core::state::{IntoText, IntoValue, callback};
use kivact_core::style::style;
use kivact_core::view::HandlerResult;
use kivact_core::view::{Element, Node, column, node, scroll, text};
use kivact_core::view::{
    Handled, IntoView, NodeRef, Placement, PopupKind, View, for_each, overlay, show,
};
use kivact_interaction::{ListItem, ListItemState, PopupState};
use std::rc::Rc;

/// What an option row shows, from its item and its state.
type OptionContent<K> = kivact_core::view::ViewFn<(Value<ListItem<K>>, Value<ListItemState>)>;

/// Recipe variant supplied to options mounted in a select's touch sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectSheetOption(pub ControlSize);

impl Variant for SelectSheetOption {}

type TouchSheet = (Rc<dyn Fn() -> bool>, SheetFactory);
type OptionRows = Rc<dyn Fn(bool) -> View>;
type SheetContent = kivact_core::view::ViewFn;
type SheetFactory = Rc<dyn Fn(Value<String>, Binding<bool>, NodeRef, SheetContent) -> View>;

pub struct Select<K: kivact_core::state::ItemKey> {
    root: Node,
    settings: SelectOptions<K>,
}

struct SelectOptions<K: kivact_core::state::ItemKey> {
    label: Value<String>,
    options: Value<Vec<ListItem<K>>>,
    selected: Binding<Option<K>>,
    on_edit: Option<kivact_core::state::Callback<kivact_core::view::EditPhase>>,
    placeholder: Option<Value<String>>,
    preview: Option<Callback<Option<K>>>,
    icon: Option<View>,
    size: Value<ControlSize>,
    option: Option<OptionContent<K>>,
    variants: Variants,
    arrow_open: Value<bool>,
    keep_trigger_focus: Value<bool>,
    touch_sheet: Option<TouchSheet>,
}
/// A themed keyed select whose value, focus and keys belong to interaction.
///
/// The trigger shows the selected option's label and the theme's
/// `chevron-down` icon; an option row shows its icon, its label over its
/// description and, while it is the selected one, the theme's `check` icon.
/// [`option`](Select::item) replaces what an option row shows, the check
/// included.
pub fn select<K: kivact_core::state::ItemKey>(
    label: impl IntoText,
    items: impl IntoValue<Vec<ListItem<K>>>,
    selected: impl kivact_core::state::IntoBinding<Option<K>>,
) -> Select<K> {
    let label = label.into_text_value();
    let variants = Variants::default();
    Select {
        root: node().name(label),
        settings: SelectOptions {
            label,
            options: items.into_value(),
            selected: selected.into_binding(),
            placeholder: None,
            on_edit: None,
            preview: None,
            icon: None,
            size: crate::values::default_control_size().into_value(),
            option: None,
            variants,
            arrow_open: false.into_value(),
            keep_trigger_focus: true.into_value(),
            touch_sheet: None,
        },
    }
}
impl<K: kivact_core::state::ItemKey> Select<K> {
    /// Observe the beginning and commit of each value edit.
    pub fn on_edit<R: kivact_core::view::HandlerResult>(
        mut self,
        f: impl Fn(kivact_core::view::EditPhase) -> R + 'static,
    ) -> Self {
        self.settings.on_edit = Some(kivact_core::state::callback(f));
        self
    }

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
    pub fn variant(mut self, v: impl Variant) -> Self {
        self.settings.variants.add(v);
        self
    }
}
impl<K: kivact_core::state::ItemKey> Select<K> {
    /// Open the options on a bare ArrowDown or ArrowUp without selecting.
    pub fn open_on_arrow(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.settings.arrow_open = enabled.into_value();
        self
    }

    /// Keep keyboard focus on the trigger while highlighting options.
    pub fn keep_trigger_focus(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.settings.keep_trigger_focus = enabled.into_value();
        self
    }

    /// Choose the surface used for touch activation. Keyboard and mouse keep
    /// the anchored list; adaptive presentation follows touch capabilities.
    pub fn presentation(self, presentation: crate::SelectPresentation) -> Self {
        self.touch_sheet(
            move || presentation.uses_sheet(),
            |label, open, _anchor, content| {
                crate::dialog(label, open)
                    .mode(kivact_interaction::DialogMode::Drawer(
                        kivact_core::geometry::Edge::Bottom,
                    ))
                    .content(move || content.call(()))
            },
        )
    }

    /// Present touch taps in an alternate modal surface when eligible.
    /// Mouse and keyboard always use the regular popup. The modal factory
    /// receives the label, its open binding, trigger ref, and listbox builder.
    pub fn touch_sheet<V: IntoView>(
        mut self,
        eligible: impl Fn() -> bool + 'static,
        factory: impl Fn(Value<String>, Binding<bool>, NodeRef, kivact_core::view::ViewFn) -> V
        + 'static,
    ) -> Self {
        self.settings.touch_sheet = Some((
            Rc::new(eligible),
            Rc::new(move |label, open, anchor, content| {
                factory(label, open, anchor, content).into_view()
            }),
        ));
        self
    }

    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.settings.placeholder = Some(text.into_text_value());
        self
    }
    pub fn on_active_change<R: HandlerResult>(
        mut self,
        preview: impl Fn(Option<K>) -> R + 'static,
    ) -> Self {
        self.settings.preview = Some(callback(preview));
        self
    }
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.settings.icon = Some(icon.into_view());
        self
    }
    /// The size (default [`ControlSize::Medium`]), given to the trigger
    /// recipe in [`SelectTriggerState::size`]; the recipe sizes the trigger
    /// by it.
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.settings.size = size.into_value();
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
    ///     select("Fruit", options, chosen).item(|item, state| {
    ///         text(move || {
    ///             let mark = if state.get().selected { "* " } else { "" };
    ///             format!("{mark}{}", item.get().label())
    ///         })
    ///     })
    /// }
    /// ```
    pub fn item<V: IntoView>(
        mut self,
        content: impl Fn(Value<ListItem<K>>, Value<ListItemState>) -> V + 'static,
    ) -> Self {
        self.settings.option = Some(kivact_core::view::ViewFn::from(move |(item, state)| {
            content(item, state)
        }));
        self
    }
}
impl<K: kivact_core::state::ItemKey> Element for Select<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl<K: kivact_core::state::ItemKey> IntoView for Select<K> {
    fn into_view(mut self) -> View {
        let behavior = configure_select(&self.settings);
        let anchor = kivact_core::view::node_ref();
        let shown = behavior.is_open();
        let open = behavior.open_binding();
        let sheet_open = behavior.is_sheet_open();
        let (trigger, sheet_factory) = select_trigger(
            behavior,
            &mut self.root,
            anchor,
            self.settings.touch_sheet.take(),
        );

        let state = trigger.state();
        let (options, selected, size) = (
            self.settings.options,
            self.settings.selected.into_value(),
            self.settings.size,
        );
        let trigger_state = trigger_projection(state, options, selected, size);
        self.root = with_base_recipe_style(
            self.root,
            |r| r.select_recipes().trigger_recipe(),
            &self.settings.variants,
            move || trigger_state.get(),
        );
        let name = self.root.accessible_name();
        let fallback = self.settings.label;
        let label = kivact_core::state::memo(move || {
            name.as_ref()
                .map(|read| read.get())
                .unwrap_or_else(|| fallback.get())
        });
        let variants = self.settings.variants;
        let indicator = with_recipe_style(
            node(),
            |r| r.select_recipes().indicator_recipe(),
            &variants,
            move || state.get(),
        )
        .child(kivact_core::view::icon("chevron-down"));
        let trigger = self
            .root
            .node_ref(anchor)
            .part(trigger)
            .child(self.settings.icon)
            .child(text(behavior.label()).style(style().grow(1).single_line()))
            .child(indicator);
        let build = option_rows(
            behavior,
            self.settings.options,
            self.settings.option,
            size,
            variants.clone(),
        );
        let surface = popup_surface(behavior, anchor, label, variants, build.clone());
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
                kivact_core::view::ViewFn::from(move |()| {
                    column(build(true))
                        .part(behavior.sheet_content())
                        .name(label)
                }),
            )
        });
        (trigger, surface, sheet).into_view()
    }
}

/// Apply sheet size variants during styling so live sizes preserve row identity.
fn with_option_style(
    root: Node,
    pick: fn(&crate::recipes::UiRecipes) -> &crate::recipes::Recipe<ListItemState>,
    variants: &Variants,
    state: Value<ListItemState>,
    size: Value<ControlSize>,
    sheet: bool,
) -> Node {
    let variants = variants.clone();
    root.style(move || {
        let variants = if sheet {
            variants.with(SelectSheetOption(size.get()))
        } else {
            variants.clone()
        };
        let theme = kivact_core::style::use_theme().get();
        pick(&crate::layer::recipes(&theme))(&theme, &variants.look(state.get()))
    })
}

/// Apply independent interaction configuration before constructing select parts.
fn configure_select<K: kivact_core::state::ItemKey>(
    settings: &SelectOptions<K>,
) -> kivact_interaction::SelectApi<K> {
    let mut behavior = kivact_interaction::use_select(settings.options, settings.selected)
        .open_on_arrow(settings.arrow_open)
        .keep_trigger_focus(settings.keep_trigger_focus);
    if let Some(observer) = settings.on_edit {
        behavior = behavior.on_edit(move |phase| observer.call(phase));
    }
    if let Some(text) = settings.placeholder {
        behavior = behavior.placeholder(text);
    }
    if let Some(preview) = settings.preview {
        behavior = behavior.on_active_change(move |key| preview.try_call(key));
    }
    behavior
}

/// Recognize touch-sheet activation while preserving the canonical trigger part.
fn select_trigger<K: kivact_core::state::ItemKey>(
    behavior: kivact_interaction::SelectApi<K>,
    root: &mut Node,
    anchor: NodeRef,
    touch_sheet: Option<TouchSheet>,
) -> (kivact_core::custom::Part<PopupState>, Option<SheetFactory>) {
    let shown = behavior.is_open();
    let sheet_open = behavior.is_sheet_open();
    if let Some((eligible, factory)) = touch_sheet {
        let disabled = root.accessible_disabled();
        let bounds = anchor.rect();
        let gesture = kivact_core::state::machine(
            machine::SheetGesture::Idle,
            move || machine::Machine {
                readiness: if shown.get() || disabled.as_ref().is_some_and(|read| read.get()) {
                    machine::Readiness::Unavailable
                } else {
                    machine::Readiness::Ready
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
        let trigger = behavior.trigger_with_sheet(move || {
            let tap = matches!(
                gesture.get(),
                machine::SheetGesture::Released(machine::Outcome::Tap)
            );
            gesture.send(machine::Event::Reset);
            tap && eligible()
        });
        let trigger_state = trigger.state();
        let trigger = kivact_core::custom::Part::new(trigger_state, move |host| {
            host.part(trigger)
                .on_pointer(move |event| {
                    gesture.send(machine::Event::Pointer(event));
                    Handled::No
                })
                .on_key(move |_| {
                    gesture.send(machine::Event::Reset);
                    Handled::No
                })
                .has_popup(move || {
                    Some(if sheet_open.get() {
                        PopupKind::Dialog
                    } else {
                        PopupKind::ListBox
                    })
                })
        });
        (trigger, Some(factory))
    } else {
        (behavior.trigger(), None)
    }
}

/// Build the same keyed options for popup and sheet presentation.
fn option_rows<K: kivact_core::state::ItemKey>(
    behavior: kivact_interaction::SelectApi<K>,
    options: Value<Vec<ListItem<K>>>,
    slot: Option<OptionContent<K>>,
    size: Value<ControlSize>,
    variants: Variants,
) -> OptionRows {
    let body_variants = variants.clone();
    Rc::new(move |sheet: bool| {
        let slot = slot.clone();
        let rows = body_variants.clone();
        for_each(
            options,
            |item| item.key().clone(),
            move |item| {
                let key = item.with_untracked(|item| item.key().clone());
                let variants = &rows;
                let lines = rows.clone();
                let part = if sheet {
                    behavior.sheet_item(key)
                } else {
                    behavior.item(key)
                };
                let state = part.state();
                let content = match &slot {
                    Some(slot) => slot.call((item, state)),
                    None => (
                        crate::helpers::item_icon(move || item.get().icon().cloned()),
                        crate::helpers::label_and_description(
                            move || item.get().label(),
                            move || item.get().description(),
                            move |line| {
                                with_option_style(
                                    line,
                                    |r| r.select_recipes().description_recipe(),
                                    &lines,
                                    state,
                                    size,
                                    sheet,
                                )
                            },
                        ),
                        with_option_style(
                            node(),
                            |r| r.select_recipes().check_recipe(),
                            variants,
                            state,
                            size,
                            sheet,
                        )
                        .child(kivact_core::view::dynamic(
                            move || state.get().selected,
                            |selected| selected.then(|| kivact_core::view::icon("check")),
                        )),
                    )
                        .into_view(),
                };
                with_option_style(
                    node().part(part),
                    |r| r.select_recipes().option_recipe(),
                    variants,
                    state,
                    size,
                    sheet,
                )
                .child(content)
            },
        )
        .into_view()
    })
}

/// Mount the anchored options only while the popup surface is selected.
fn popup_surface<K: kivact_core::state::ItemKey>(
    behavior: kivact_interaction::SelectApi<K>,
    anchor: NodeRef,
    label: Value<String>,
    variants: Variants,
    popup_build: OptionRows,
) -> View {
    let shown = behavior.is_open();
    let sheet_open = behavior.is_sheet_open();
    show(
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
            .with_node(|node| node.part(part))
            .name(label)
            .offset(4.0)
            .at_least_anchor_width()
        },
    )
    .into_view()
}

/// Project placeholder availability from the actual selected key and option source.
fn trigger_projection<K: kivact_core::state::ItemKey>(
    state: Value<PopupState>,
    options: Value<Vec<ListItem<K>>>,
    selected: Value<Option<K>>,
    size: Value<ControlSize>,
) -> Value<SelectTriggerState> {
    kivact_core::state::memo(move || {
        let PopupState { input, open } = state.get();
        let placeholder = selected.with(|key| {
            key.as_ref()
                .is_none_or(|key| options.with(|rows| rows.iter().all(|row| row.key() != key)))
        });
        SelectTriggerState {
            input,
            open,
            placeholder,
            size: size.get(),
        }
    })
}
