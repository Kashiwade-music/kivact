//! Role: Render keyed breadcrumb navigation with one current page.
use crate::BreadcrumbItemState;
use kivact_core::state::{IntoValue, Value};
use kivact_core::view::{CurrentItem, Element, IntoView, Node, Role, View, dynamic, node, text};
use kivact_interaction::ListItem;
use std::rc::Rc;

/// Immutable choice of the source used to project the current page.
enum CurrentPage<K: 'static> {
    Last,
    Explicit(Value<Option<K>>),
}

type ItemSlot<K> = Rc<dyn Fn(ListItem<K>) -> View>;
/// A keyed navigation trail. The final item is the current page.
pub struct Breadcrumb<K: 'static> {
    root: Node,
    items: Value<Vec<ListItem<K>>>,
    current: CurrentPage<K>,
    activate: Option<Rc<dyn Fn(K)>>,
    item: Option<ItemSlot<K>>,
    separator: Rc<dyn Fn() -> View>,
    variants: crate::variants::Variants,
}
/// Construct a breadcrumb trail whose last item has `CurrentItem::Page`.
#[track_caller]
pub fn breadcrumb<K: Clone + PartialEq + 'static>(
    items: impl IntoValue<Vec<ListItem<K>>>,
) -> Breadcrumb<K> {
    Breadcrumb {
        root: node()
            .role(Role::Navigation)
            .name(kivact_core::i18n::message("breadcrumb.label", "Breadcrumb")),
        items: items.into_value(),
        current: CurrentPage::Last,
        activate: None,
        item: None,
        separator: Rc::new(|| text("/").into_view()),
        variants: crate::variants::Variants::default(),
    }
}
impl<K: Clone + PartialEq + 'static> Breadcrumb<K> {
    /// Override the current page by key. `None` marks no item as current.
    /// Without this override, the final item is current.
    pub fn current(mut self, current: impl IntoValue<Option<K>>) -> Self {
        self.current = CurrentPage::Explicit(current.into_value());
        self
    }
    /// Activate a non-current item by key.
    pub fn on_activate(mut self, f: impl Fn(K) + 'static) -> Self {
        self.activate = Some(Rc::new(f));
        self
    }
    /// Replace the visible contents of each item, retaining its semantics.
    pub fn item<V: IntoView>(mut self, f: impl Fn(ListItem<K>) -> V + 'static) -> Self {
        self.item = Some(Rc::new(move |item| f(item).into_view()));
        self
    }
    /// Replace the decorative separator between items.
    pub fn separator<V: IntoView>(mut self, f: impl Fn() -> V + 'static) -> Self {
        self.separator = Rc::new(move || f().into_view());
        self
    }
    /// Add a kit-defined visual variant.
    pub fn variant(mut self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl<K: 'static> Element for Breadcrumb<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl<K: Clone + PartialEq + 'static> IntoView for Breadcrumb<K> {
    fn into_view(self) -> View {
        let Self {
            root,
            items,
            current,
            activate,
            item,
            separator,
            variants,
        } = self;
        let child_variants = variants.clone();
        let source = kivact_core::state::memo(move || {
            let items = items.get();
            let current = match &current {
                CurrentPage::Last => items.last().map(|item| item.key().clone()),
                CurrentPage::Explicit(value) => value.get(),
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
                            |r| r.breadcrumb_recipes().separator_recipe(),
                            &child_variants,
                            || (),
                        )
                        .child(separator())
                        .into_view(),
                    );
                }
                let current = current.as_ref() == Some(entry.key());
                let entry_node = breadcrumb_entry(
                    entry,
                    current,
                    activate.as_ref(),
                    item.as_ref(),
                    &child_variants,
                );
                children.push(node().role(Role::ListItem).child(entry_node).into_view());
            }
            children
        });
        crate::layer::with_base_recipe_style(
            root,
            |r| r.breadcrumb_recipes().root_recipe(),
            &variants,
            || (),
        )
        .child(
            crate::layer::with_recipe_style(
                node().role(Role::List),
                |r| r.breadcrumb_recipes().list_recipe(),
                &variants,
                || (),
            )
            .child(contents),
        )
        .into_view()
    }
}

/// Render a current-page label or an activatable navigation link.
fn breadcrumb_entry<K: Clone + PartialEq + 'static>(
    entry: ListItem<K>,
    current: bool,
    activate: Option<&Rc<dyn Fn(K)>>,
    item: Option<&ItemSlot<K>>,
    variants: &crate::variants::Variants,
) -> View {
    let label_entry = entry.clone();
    let label = kivact_core::state::memo(move || label_entry.label());
    let body = item.map_or_else(|| text(label).into_view(), |slot| slot(entry.clone()));
    if current || activate.is_none() {
        let current = current.then_some(CurrentItem::Page);
        if item.is_none() {
            crate::layer::with_recipe_style(
                text(label).current_item(current),
                |r| r.breadcrumb_recipes().item_recipe(),
                variants,
                move || BreadcrumbItemState {
                    current: current.is_some(),
                    input: None,
                },
            )
            .into_view()
        } else {
            crate::layer::with_recipe_style(
                node()
                    .role(Role::Generic)
                    .name(label)
                    .current_item(current)
                    .child(body),
                |r| r.breadcrumb_recipes().item_recipe(),
                variants,
                move || BreadcrumbItemState {
                    current: current.is_some(),
                    input: None,
                },
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
                        f(key.clone());
                    }
                })
                .child(body),
            |r| r.breadcrumb_recipes().item_recipe(),
            variants,
            move || BreadcrumbItemState {
                current,
                input: Some(input.get()),
            },
        )
        .into_view()
    }
}
