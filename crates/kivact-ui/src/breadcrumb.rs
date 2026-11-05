//! Role: Render keyed breadcrumb navigation with one current page.
use kivact_core::state::{IntoValue, Value};
use kivact_core::view::{
    CurrentItem, IntoView, Node, NodeBuilder, Role, View, dynamic, node, text,
};
use kivact_interaction::ListItem;
use std::rc::Rc;

type ItemSlot<K> = Rc<dyn Fn(ListItem<K>) -> View>;
/// A keyed navigation trail. The final item is the current page.
pub struct Breadcrumb<K: 'static> {
    root: Node,
    items: Value<Vec<ListItem<K>>>,
    // xtask-allow(state-fields): Optional construction-time activation callback, independent of the presentation slot.
    activate: Option<Rc<dyn Fn(K)>>,
    // xtask-allow(state-fields): Optional presentation slot selected during construction; breadcrumbs store no interaction state.
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
        activate: None,
        item: None,
        separator: Rc::new(|| text("/").into_view()),
        variants: crate::variants::Variants::default(),
    }
}
impl<K: Clone + PartialEq + 'static> Breadcrumb<K> {
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
    pub fn variant(self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl<K: 'static> NodeBuilder for Breadcrumb<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl<K: Clone + PartialEq + 'static> IntoView for Breadcrumb<K> {
    fn into_view(self) -> View {
        let Self {
            root,
            items,
            activate,
            item,
            separator,
            variants,
        } = self;
        let child_variants = variants.clone();
        let contents = dynamic(items, move |items| {
            let last = items.len().saturating_sub(1);
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
                let current = at == last;
                let entry_node = breadcrumb_entry(entry, current, &activate, item.as_ref());
                children.push(
                    crate::layer::with_recipe_style(
                        node().role(Role::ListItem),
                        |r| r.breadcrumb_recipes().item_recipe(),
                        &child_variants,
                        move || current,
                    )
                    .child(entry_node)
                    .into_view(),
                );
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
            node()
                .role(Role::List)
                .style(
                    kivact_core::style::style()
                        .direction(kivact_core::style::Direction::Row)
                        .gap(8),
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
    activate: &Option<Rc<dyn Fn(K)>>,
    item: Option<&ItemSlot<K>>,
) -> View {
    let label_entry = entry.clone();
    let label = kivact_core::state::memo(move || label_entry.label());
    let body = item.map_or_else(|| text(label).into_view(), |slot| slot(entry.clone()));
    if current || activate.is_none() {
        let current = current.then_some(CurrentItem::Page);
        if item.is_none() {
            text(label).current_item(current).into_view()
        } else {
            node()
                .role(Role::Generic)
                .name(label)
                .current_item(current)
                .child(body)
                .into_view()
        }
    } else {
        let f = activate.clone();
        let key = entry.key().clone();
        crate::button("")
            .name(label)
            .with_node(|n| n.role(Role::Link))
            .disabled(entry.is_disabled())
            .on_press(move || {
                if let Some(f) = &f {
                    f(key.clone());
                }
            })
            .child(body)
            .into_view()
    }
}
