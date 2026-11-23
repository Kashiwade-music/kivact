//! Role: The breadcrumb component, its item entries and separators.

use crate::breadcrumb::look::BreadcrumbTheme;
use crate::theme::BreadcrumbItemState;
use kivact_core::custom::Caller;
use kivact_core::state::{Callback, Value, memo};
use kivact_core::view::{CurrentItem, IntoView, Role, View, ViewFn, dynamic, node, text};
use kivact_interaction::ListItem;
use kivact_macros::component;
use std::rc::Rc;

type ItemSlot<K> = Rc<dyn Fn(ListItem<K>) -> View>;

/// A keyed navigation trail. The final item is the current page; `current`
/// overrides it by key (`None` marks no item as current).
///
/// `on_activate` runs with the key of a non-current item when it is pressed.
/// `item` replaces the visible contents of each item, keeping its semantics,
/// and `separator` replaces the decorative mark between items.
#[component]
pub fn breadcrumb<K: Clone + PartialEq + 'static>(
    items: Value<Vec<ListItem<K>>>,
    current: Option<Value<Option<K>>>,
    on_activate: Option<Callback<K>>,
    item: Option<Rc<dyn Fn(ListItem<K>) -> View>>,
    separator: Option<ViewFn>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let child_variants = variants.clone();
    let separator = separator.unwrap_or_else(|| ViewFn::from(|()| text("/")));
    let source = memo(move || {
        let items = items.get();
        let current = match &current {
            None => items.last().map(|item| item.key().clone()),
            Some(value) => value.get(),
        };
        (items, current)
    });
    let contents = dynamic(source, move |(items, current)| {
        let mut children = Vec::new();
        for (at, entry) in items.into_iter().enumerate() {
            if at > 0 {
                children.push(
                    crate::layer::with_recipe_style(
                        node(),
                        |r: &BreadcrumbTheme| r.separator_recipe(),
                        &child_variants,
                        || (),
                    )
                    .child(separator.call(()))
                    .into_view(),
                );
            }
            let current = current.as_ref() == Some(entry.key());
            let entry_node = breadcrumb_entry(
                entry,
                current,
                on_activate.as_ref(),
                item.as_ref(),
                &child_variants,
            );
            children.push(node().role(Role::ListItem).child(entry_node).into_view());
        }
        children
    });
    crate::layer::with_base_recipe_style(
        node()
            .role(Role::Navigation)
            .name(kivact_core::i18n::message("breadcrumb.label", "Breadcrumb")),
        |r: &BreadcrumbTheme| r.root_recipe(),
        &variants,
        || (),
    )
    .child(
        crate::layer::with_recipe_style(
            node().role(Role::List),
            |r: &BreadcrumbTheme| r.list_recipe(),
            &variants,
            || (),
        )
        .child(contents),
    )
}

/// Render a current-page label or an activatable navigation link.
fn breadcrumb_entry<K: Clone + PartialEq + 'static>(
    entry: ListItem<K>,
    current: bool,
    activate: Option<&Callback<K>>,
    item: Option<&ItemSlot<K>>,
    variants: &kivact_core::view::Variants,
) -> View {
    let label_entry = entry.clone();
    let label = kivact_core::state::memo(move || label_entry.label());
    let body = item.map_or_else(|| text(label).into_view(), |slot| slot(entry.clone()));
    if current || activate.is_none() {
        let current = current.then_some(CurrentItem::Page);
        let state = if current.is_some() {
            BreadcrumbItemState::Current
        } else {
            BreadcrumbItemState::Label
        };
        if item.is_none() {
            crate::layer::with_recipe_style(
                text(label).current_item(current),
                |r: &BreadcrumbTheme| r.item_recipe(),
                variants,
                move || state,
            )
            .into_view()
        } else {
            crate::layer::with_recipe_style(
                node()
                    .role(Role::Generic)
                    .name(label)
                    .current_item(current)
                    .child(body),
                |r: &BreadcrumbTheme| r.item_recipe(),
                variants,
                move || state,
            )
            .into_view()
        }
    } else {
        let f = activate.cloned();
        let key = entry.key().clone();
        let part = kivact_interaction::use_button()
            .disabled(entry.is_disabled())
            .control();
        let input = part.state();
        crate::layer::with_recipe_style(
            node()
                .part(part)
                .name(label)
                .role(Role::Link)
                .on_press(move || {
                    if let Some(f) = &f {
                        f.call(key.clone());
                    }
                })
                .child(body),
            |r: &BreadcrumbTheme| r.item_recipe(),
            variants,
            move || BreadcrumbItemState::Link(input.get()),
        )
        .into_view()
    }
}
