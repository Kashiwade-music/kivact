//! Role: Render themed menus recursively over shared menu level parts.
//!
//! One renderer draws both the ordinary trigger's placement and the context
//! placement at the pointer. The builder draws its default trigger button;
//! `.attach(trigger)` takes the caller's trigger and `.attach_context(target)`
//! the right-click target, both `impl Element + IntoView`. A trigger that
//! has its own role (a menu bar's `MenuItem`, read with
//! `Node::accessible_role`) keeps it after attaching; one without becomes a
//! `Button`. The root surface's side and alignment are `.placement` and
//! `.align`, `.at_least_anchor_width` makes it as wide as the trigger, and
//! submenus are drawn with the `sub_surface` recipe. An
//! [`inset`](kivact_interaction::MenuEntry::inset) item or heading puts the
//! `inset` recipe's box before its content so it lines up with the check and
//! radio items. The leading check box and the trailing submenu box are
//! separate slots of the recipe table, and both get the item's actual
//! `MenuItemState`; the meaning of a mark is never switched by faking the
//! state. A menu bar uses the headless menu parts to coordinate its triggers.
use super::layer::with_recipe_style;
use super::variants::{Variant, Variants};
use kivact_core::state::Value;
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::view::HandlerResult;
use kivact_core::view::{Element, Node, column, node, scroll, text};
use kivact_core::view::{
    IntoView, NodeRef, OverlayAlign, Placement, View, for_each, overlay, overlay_at,
};
use kivact_interaction::{
    MenuApi as Behavior, MenuCheck, MenuEntry, MenuItemKind, MenuItemState, MenuOrigin, MenuRow,
};

/// What an item row shows, from its entry and its state.
type ItemContent<K> = kivact_core::view::ViewFn<(Value<MenuEntry<K>>, Value<MenuItemState>)>;

pub struct Menu<K: kivact_core::state::ItemKey> {
    label: Value<String>,
    behavior: Behavior<K>,
    // xtask-allow(state-fields): option: absent means the component's default
    offset: Option<f64>,
    placement: Placement,
    // xtask-allow(state-fields): option: absent means the component's default
    align: Option<OverlayAlign>,
    // xtask-allow(state-fields): option: Menu::at_least_anchor_width; false sizes the surface to its content
    at_least_anchor_width: bool,
    // xtask-allow(state-fields): option: absent means the component's default
    item: Option<ItemContent<K>>,
    variants: Variants,
}
/// A themed action menu; its recursive behavior remains entirely headless.
///
/// As a view it is a themed button named `label` that opens the menu. Open
/// it from your own trigger with [`attach`](Menu::attach), or by a context
/// press on a target with [`attach_context`](Menu::attach_context).
///
/// An item row shows its check or radio mark (the theme's `check` and
/// `menu-radio` icons), its icon, its label over its description, its shortcut and
/// a submenu's `chevron-right`; headings and the labels of groups show
/// their text, and an [`inset`](MenuEntry::inset) row keeps the theme's
/// inset box before its content. [`item`](Menu::item) replaces what an item
/// row shows.
#[track_caller]
pub fn menu<K: kivact_core::state::ItemKey>(
    label: impl IntoText,
    items: impl IntoValue<Vec<MenuEntry<K>>>,
) -> Menu<K> {
    Menu {
        label: label.into_text_value(),
        behavior: kivact_interaction::use_menu(items),
        offset: None,
        placement: Placement::Below,
        align: None,
        at_least_anchor_width: false,
        item: None,
        variants: Variants::default(),
    }
}
impl<K: kivact_core::state::ItemKey> Menu<K> {
    /// Run a handler when an item is activated.
    pub fn on_activate<R: HandlerResult>(mut self, choose: impl Fn(K) -> R + 'static) -> Self {
        self.behavior = self.behavior.on_activate(choose);
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
        self.variants.add(v);
        self
    }
}
impl<K: kivact_core::state::ItemKey> Menu<K> {
    /// Set the root surface's distance from its trigger (default 4 logical
    /// pixels; a context menu opens right at the pointer or its target, 0);
    /// submenus stay adjacent.
    pub fn offset(mut self, offset: impl Into<f64>) -> Self {
        let offset = offset.into();
        self.offset = Some(offset);
        self
    }
    /// The side of the trigger the root surface opens on (default
    /// [`Placement::Below`]); a context menu opened by the pointer opens
    /// on that side of the pointer. Submenus open at their item's end.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }
    /// Where along the trigger's side the root surface sits (the overlay's
    /// [`align`](kivact_core::view::Overlay::align); its start by default).
    pub fn align(mut self, align: OverlayAlign) -> Self {
        self.align = Some(align);
        self
    }
    /// Make the root surface at least as wide as its trigger (the overlay's
    /// [`at_least_anchor_width`](kivact_core::view::Overlay::at_least_anchor_width));
    /// a context menu ignores it. A constant: it is the shape of the layout.
    pub fn at_least_anchor_width(mut self) -> Self {
        self.at_least_anchor_width = true;
        self
    }
    /// What each item row shows, in every level, built once per row from
    /// its entry and its state (both reactive). The row itself, its keys,
    /// its semantics, its recipe and a submenu's surface stay the
    /// component's; headings and separators are drawn as usual.
    ///
    /// ```no_run
    /// use kivact::prelude::*;
    ///
    /// fn actions() -> impl IntoView {
    ///     let entries = vec![MenuEntry::item("new", "New"), MenuEntry::item("open", "Open")];
    ///     menu("File", entries).on_activate(|_key| {}).item(|entry, state| {
    ///         text(move || {
    ///             let mark = if state.get().input.is_focused() { "> " } else { "" };
    ///             format!("{mark}{}", entry.get().label())
    ///         })
    ///     })
    /// }
    /// ```
    pub fn item<V: IntoView>(
        mut self,
        content: impl Fn(Value<MenuEntry<K>>, Value<MenuItemState>) -> V + 'static,
    ) -> Self {
        self.item = Some(kivact_core::view::ViewFn::from(move |(entry, state)| {
            content(entry, state)
        }));
        self
    }
    /// Open the menu from `trigger` (a press, Enter, Space or the Down key)
    /// and return the trigger with the menu. A trigger without a name of its
    /// own is named by the menu's label; a trigger with a role keeps it (a
    /// menu bar's triggers are `Role::MenuItem`), and one without is a
    /// `Role::Button`.
    pub fn attach<T: Element + IntoView>(self, trigger: T) -> View {
        self.render(Some(trigger), Opening::Trigger)
    }
    /// Open the same themed menu by a context press or the context-menu key
    /// on `target`, at the pointer, and return the target with the menu. Its
    /// top level is drawn with the `context_surface` recipe.
    pub fn attach_context<T: Element + IntoView>(self, target: T) -> View {
        self.render(Some(target), Opening::Context)
    }
    pub(crate) fn attach_context_fallback<T: Element + IntoView>(self, target: T) -> View {
        self.render(Some(target), Opening::ContextFallback)
    }

    fn render<T: Element + IntoView>(self, trigger: Option<T>, opening: Opening) -> View {
        let part = match opening {
            Opening::ContextFallback => self.behavior.context_fallback_trigger(),
            Opening::Context => self.behavior.context_trigger(),
            Opening::Trigger => self.behavior.trigger(),
        };
        let kind = if opening == Opening::Trigger {
            LevelKind::Anchored
        } else {
            LevelKind::Context
        };
        let anchor = kivact_core::view::node_ref();
        let (name, trigger) = match trigger {
            Some(mut trigger) => {
                let trigger = if trigger.node_mut().accessible_name().is_none() {
                    trigger.name(self.label)
                } else {
                    trigger
                };
                named(trigger.node_ref(anchor), part)
            }
            None => {
                let state = part.state();
                let trigger = with_recipe_style(
                    node().name(self.label),
                    |r| r.menu_recipes().trigger_recipe(),
                    &self.variants,
                    move || state.get(),
                )
                .child(text(self.label));
                named(trigger.node_ref(anchor), part)
            }
        };
        let fallback = self.label;
        let label = kivact_core::state::memo(move || {
            name.as_ref()
                .map(|read| read.get())
                .unwrap_or_else(|| fallback.get())
        });
        (
            trigger,
            surface(
                Level {
                    behavior: self.behavior,
                    anchor,
                    placement: self.placement,
                    align: self.align,
                    at_least_anchor_width: self.at_least_anchor_width
                        && kind == LevelKind::Anchored,
                    kind,
                    label,
                    offset: self.offset.unwrap_or(if kind == LevelKind::Context {
                        0.0
                    } else {
                        4.0
                    }),
                    variants: self.variants,
                },
                self.item,
            ),
        )
            .into_view()
    }
}
/// The trigger's accessible name and the trigger with `part` attached; a
/// role the trigger has (a menu bar's `MenuItem`) stays the trigger's.
fn named<T: Element + IntoView, S: kivact_core::state::PropValue>(
    mut trigger: T,
    part: kivact_core::custom::Part<S>,
) -> (Option<Value<String>>, View) {
    let name = trigger.node_mut().accessible_name();
    let role = trigger.node_mut().accessible_role();
    let trigger = trigger.with_node(|node| node.part(part));
    let trigger = match role {
        Some(role) => trigger.with_node(|node| node.role(role)),
        None => trigger,
    };
    (name, trigger.into_view())
}
impl<K: kivact_core::state::ItemKey> IntoView for Menu<K> {
    fn into_view(self) -> View {
        self.render(None::<Node>, Opening::Trigger)
    }
}

/// How the top level of a menu is opened.
#[derive(Clone, Copy, PartialEq)]
enum Opening {
    Trigger,
    Context,
    ContextFallback,
}
/// Which surface one level of the open menu is.
#[derive(Clone, Copy, PartialEq)]
enum LevelKind {
    Anchored,
    Context,
    Submenu,
}

/// One level of the open menu: where and how its surface shows.
struct Level<K: kivact_core::state::ItemKey> {
    behavior: Behavior<K>,
    anchor: NodeRef,
    placement: Placement,
    // xtask-allow(state-fields): option: absent means the component's default
    align: Option<OverlayAlign>,
    // xtask-allow(state-fields): option: copied from Menu::at_least_anchor_width for the top level; submenu levels are false
    at_least_anchor_width: bool,
    kind: LevelKind,
    label: Value<String>,
    offset: f64,
    variants: Variants,
}

/// The theme's inset box before an inset row's content, while `entry` is
/// inset.
fn inset_box<K: 'static>(entry: Value<MenuEntry<K>>, variants: &Variants) -> View {
    let variants = variants.clone();
    kivact_core::view::show(
        move || entry.with(|entry| entry.is_inset()),
        move || {
            let part = kivact_core::custom::part(|state| state);
            let state = part.state();
            with_recipe_style(
                node().part(part),
                |r| r.menu_recipes().inset_recipe(),
                &variants,
                move || state.get(),
            )
        },
    )
    .into_view()
}

/// The default content of an item row: its mark, icon, label and
/// description, shortcut and submenu chevron.
fn item_content<K: Clone + 'static>(
    entry: Value<MenuEntry<K>>,
    state: Value<MenuItemState>,
    variants: &Variants,
) -> View {
    let checkable = entry.with_untracked(|e| e.check().is_some());
    let submenu = entry.with_untracked(|e| e.item_kind() == Some(MenuItemKind::Submenu));
    let mark = checkable.then(|| {
        let shown = move || match entry.get().check() {
            Some(MenuCheck::Radio(true)) => Some("menu-radio"),
            Some(MenuCheck::Checkbox(true)) => Some("check"),
            Some(MenuCheck::Radio(false) | MenuCheck::Checkbox(false)) | None => None,
        };
        with_recipe_style(
            node(),
            |r| r.menu_recipes().check_recipe(),
            variants,
            move || state.get(),
        )
        .child(kivact_core::view::dynamic(shown, |name| {
            name.map(|name| kivact_core::view::icon(name))
        }))
    });
    let chevron = submenu.then(|| {
        with_recipe_style(
            node(),
            |r| r.menu_recipes().submenu_recipe(),
            variants,
            move || state.get(),
        )
        .child(kivact_core::view::icon("chevron-right"))
    });
    let shortcuts = variants.clone();
    let lines = variants.clone();
    let shortcut = kivact_core::view::dynamic(
        move || entry.get().shortcut().to_owned(),
        move |text_value| {
            (!text_value.is_empty()).then(|| {
                with_recipe_style(
                    node(),
                    |r| r.menu_recipes().shortcut_recipe(),
                    &shortcuts,
                    move || state.get(),
                )
                .child(text(text_value))
            })
        },
    );
    (
        mark,
        super::helpers::item_icon(move || entry.get().icon().cloned()),
        super::helpers::label_and_description(
            move || entry.get().label(),
            move || entry.get().description(),
            move |line| {
                with_recipe_style(
                    line,
                    |r| r.menu_recipes().description_recipe(),
                    &lines,
                    move || state.get(),
                )
            },
        ),
        shortcut,
        chevron,
    )
        .into_view()
}

fn surface<K: kivact_core::state::ItemKey>(level: Level<K>, slot: Option<ItemContent<K>>) -> View {
    let Level {
        behavior,
        anchor,
        placement,
        align,
        at_least_anchor_width,
        kind,
        label,
        offset,
        variants,
    } = level;
    behavior.show(move || {
        let part = behavior.content();
        let state = part.state();
        let rows = menu_rows(behavior, slot.clone(), variants.clone());
        let content = with_recipe_style(
            scroll(column(rows)),
            match kind {
                LevelKind::Submenu => |r| r.menu_recipes().sub_surface_recipe(),
                LevelKind::Context => |r| r.menu_recipes().context_surface_recipe(),
                LevelKind::Anchored => |r| r.menu_recipes().surface_recipe(),
            },
            &variants,
            move || state.get(),
        );
        let origin = behavior.origin();
        let layer = if kind == LevelKind::Context
            && matches!(origin.get_untracked(), Some(MenuOrigin::Pointer(_)))
        {
            overlay_at(
                move || match origin.get() {
                    Some(MenuOrigin::Pointer(point)) => point,
                    _ => kivact_core::geometry::Point::new(0.0, 0.0),
                },
                placement,
                content,
            )
        } else {
            {
                let layer = overlay(anchor, placement, content);
                if at_least_anchor_width {
                    layer.at_least_anchor_width()
                } else {
                    layer
                }
            }
        };
        let layer = match align {
            Some(align) => layer.align(align),
            None => layer,
        };
        layer
            .with_node(|node| node.part(part))
            .name(label)
            .offset(offset)
    })
}

/// Render the keyed rows belonging to one actual menu level.
fn menu_rows<K: kivact_core::state::ItemKey>(
    behavior: Behavior<K>,
    slot: Option<ItemContent<K>>,
    variants: Variants,
) -> View {
    let looks = variants;
    let retain_disabled_focus = kivact_core::style::use_theme()
        .get()
        .ext::<kivact_interaction::InteractionPolicy>()
        .menu
        .disabled_focus
        == kivact_interaction::DisabledMenuFocus::Retain;
    for_each(
        behavior.entries(),
        |(key, _)| key.clone(),
        move |entry| {
            let row = entry.with_untracked(|entry| entry.0.clone());
            match row {
                MenuRow::Item { key, kind } => menu_item(
                    behavior,
                    key,
                    kind == MenuItemKind::Submenu,
                    entry,
                    slot.clone(),
                    &looks,
                    retain_disabled_focus,
                ),
                MenuRow::Label(_) => {
                    let part = kivact_core::custom::part(|state| state);
                    let state = part.state();
                    let data = kivact_core::state::memo(move || entry.get().1);
                    with_recipe_style(
                        kivact_core::view::row((
                            inset_box(data, &looks),
                            text(move || entry.get().1.label()),
                        ))
                        .part(part),
                        |r| r.menu_recipes().label_recipe(),
                        &looks,
                        move || state.get(),
                    )
                    .into_view()
                }
                MenuRow::Separator(_) => {
                    let part = kivact_core::custom::part(|state| state);
                    let state = part.state();
                    with_recipe_style(
                        node().part(part).role(kivact_core::view::Role::Separator),
                        |r| r.menu_recipes().separator_recipe(),
                        &looks,
                        move || state.get(),
                    )
                    .into_view()
                }
            }
        },
    )
    .into_view()
}

/// Render one enabled or disabled item and its optional recursive submenu.
fn menu_item<K: kivact_core::state::ItemKey>(
    behavior: Behavior<K>,
    key: K,
    submenu: bool,
    entry: Value<(MenuRow<K>, MenuEntry<K>)>,
    slot: Option<ItemContent<K>>,
    looks: &Variants,
    retain_disabled_focus: bool,
) -> View {
    let part = behavior.item(key.clone());
    let item_state = part.state();
    let anchor = kivact_core::view::node_ref();
    let data = kivact_core::state::memo(move || entry.get().1);
    let content = match &slot {
        Some(slot) => slot.call((data, item_state)),
        None => item_content(data, item_state, looks),
    };
    let row = with_recipe_style(
        node()
            .node_ref(anchor)
            .part(part)
            .child(inset_box(data, looks))
            .child(content),
        |r| r.menu_recipes().item_recipe(),
        looks,
        move || item_state.get(),
    );
    let row = if retain_disabled_focus {
        let reason = kivact_core::state::memo(move || {
            data.with(|entry| {
                if entry.is_disabled() {
                    entry.disabled_reason()
                } else {
                    None
                }
            })
        });
        crate::tooltip(move || reason.get().unwrap_or_default())
            .enabled(move || {
                reason.with(|reason| reason.as_ref().is_some_and(|reason| !reason.is_empty()))
            })
            .placement(Placement::End)
            .attach(row)
    } else {
        row.into_view()
    };
    let child = submenu.then(|| {
        surface(
            Level {
                behavior: behavior.submenu(key),
                anchor,
                placement: Placement::End,
                align: None,
                at_least_anchor_width: false,
                kind: LevelKind::Submenu,
                label: kivact_core::state::memo(move || entry.get().1.label()),
                offset: 0.0,
                variants: looks.clone(),
            },
            slot.clone(),
        )
    });
    (row, child).into_view()
}
