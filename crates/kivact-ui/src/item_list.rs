//! Role: Build semantic static lists and configurable item rows.
use crate::layer::with_base_recipe_style;
use crate::variants::{Variant, Variants};
use kivact_core::custom::{PartState, part};
use kivact_core::state::{Callback, IntoText, Value, callback};
use kivact_core::view::{
    HandlerResult, IntoChildren, IntoView, Node, NodeBuilder, Role, View, node, node_ref, text,
};

/// Visual overrides for static lists and item rows.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ItemListTokens {
    pub gap: Option<f64>,
    pub padding: Option<f64>,
    pub row_gap: Option<f64>,
    pub description_size: Option<f64>,
}
/// A nonselectable semantic list.
pub struct ItemList {
    root: Node,
    variants: Variants,
}
/// Group item rows with list semantics, without listbox selection behavior.
#[track_caller]
pub fn item_list(children: impl IntoChildren) -> ItemList {
    ItemList {
        root: node().role(Role::List).child(children.into_views()),
        variants: Variants::default(),
    }
}
impl ItemList {
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for ItemList {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for ItemList {
    fn into_view(self) -> View {
        let part = part(|s| s);
        let state = part.state();
        with_base_recipe_style(
            self.root.part(part),
            |r| r.item_list_recipes().root_recipe(),
            &self.variants,
            move || state.get(),
        )
        .into_view()
    }
}
/// A semantic list item with optional leading media and trailing actions.
pub struct ItemRow {
    root: Node,
    title: Value<String>,
    // xtask-allow(state-fields): Independent optional construction content slot, not an interaction phase.
    title_content: Option<View>,
    // xtask-allow(state-fields): Independent optional construction content slot, not an interaction phase.
    description: Option<Value<String>>,
    // xtask-allow(state-fields): Independent optional construction content slot, not an interaction phase.
    description_content: Option<View>,
    // xtask-allow(state-fields): Independent optional construction content slot, not an interaction phase.
    media: Option<View>,
    // xtask-allow(state-fields): Independent optional construction content slot, not an interaction phase.
    actions: Option<View>,
    // xtask-allow(state-fields): Independent optional construction content slot, not an interaction phase.
    header: Option<View>,
    // xtask-allow(state-fields): Independent optional construction content slot, not an interaction phase.
    footer: Option<View>,
    // xtask-allow(state-fields): Independent optional construction callback; mounted input belongs to Part.
    press: Option<Callback<()>>,
    variants: Variants,
}
/// Create a named item row. Pressable rows retain the list-item role.
#[track_caller]
pub fn item_row(title: impl IntoText) -> ItemRow {
    let title = title.into_text_value();
    ItemRow {
        root: node().role(Role::ListItem).name(title),
        title,
        title_content: None,
        description: None,
        description_content: None,
        media: None,
        actions: None,
        header: None,
        footer: None,
        press: None,
        variants: Variants::default(),
    }
}
impl ItemRow {
    pub fn media(mut self, view: impl IntoView) -> Self {
        self.media = Some(view.into_view());
        self
    }
    pub fn actions(mut self, view: impl IntoView) -> Self {
        self.actions = Some(view.into_view());
        self
    }
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(text.into_text_value());
        self
    }
    /// Replace the visible title while keeping the accessible title.
    pub fn title_content(mut self, view: impl IntoView) -> Self {
        self.title_content = Some(view.into_view());
        self
    }
    pub fn description_content(mut self, view: impl IntoView) -> Self {
        self.description_content = Some(view.into_view());
        self
    }
    pub fn header(mut self, view: impl IntoView) -> Self {
        self.header = Some(view.into_view());
        self
    }
    pub fn footer(mut self, view: impl IntoView) -> Self {
        self.footer = Some(view.into_view());
        self
    }
    pub fn on_press<R: HandlerResult>(mut self, press: impl Fn() -> R + 'static) -> Self {
        self.press = Some(callback(move |()| press()));
        self
    }
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for ItemRow {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for ItemRow {
    fn into_view(self) -> View {
        let Self {
            mut root,
            title,
            title_content,
            description,
            description_content,
            media,
            actions,
            header,
            footer,
            press,
            variants,
        } = self;
        if let Some(press) = press {
            root = root.focusable(true).on_press(move || press.call(()));
        }
        let part = part(|s| s);
        let state = part.state();
        let wrap = |view: View, pick: fn(&crate::UiRecipes) -> &crate::Recipe<PartState>| {
            with_base_recipe_style(node().child(view), pick, &variants, move || state.get())
        };
        let title = wrap(
            title_content.unwrap_or_else(|| text(title).into_view()),
            |r| r.item_list_recipes().title_recipe(),
        );
        let description_ref = node_ref();
        let description = description_content
            .or_else(|| description.map(|d| text(d).into_view()))
            .map(|d| {
                wrap(d, |r| r.item_list_recipes().description_recipe()).node_ref(description_ref)
            });
        if description.is_some() {
            root = root.described_by(description_ref);
        }
        let content = wrap(node().child((title, description)).into_view(), |r| {
            r.item_list_recipes().content_recipe()
        });
        let media = media.map(|v| wrap(v, |r| r.item_list_recipes().media_recipe()));
        let actions = actions.map(|v| wrap(v, |r| r.item_list_recipes().actions_recipe()));
        let header = header.map(|v| wrap(v, |r| r.item_list_recipes().header_recipe()));
        let footer = footer.map(|v| wrap(v, |r| r.item_list_recipes().footer_recipe()));
        let body = wrap(node().child((media, content, actions)).into_view(), |r| {
            r.item_list_recipes().body_recipe()
        });
        with_base_recipe_style(
            root.part(part),
            |r| r.item_list_recipes().item_recipe(),
            &variants,
            move || state.get(),
        )
        .child((header, body, footer))
        .into_view()
    }
}
