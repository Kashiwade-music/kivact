//! Role: Render rich navigation links using the closest existing panel's close request.
use crate::layer::with_recipe_style;
use crate::navigation_values::NavigationLinkState;
use crate::variants::{Variant, Variants};
use kivact_core::state::{Callback, IntoText, Value};
use kivact_core::view::{IntoView, Node, NodeBuilder, Role, View, column, node, node_ref, text};
/// A title, optional icon and description that activates and closes its navigation panel.
pub struct NavigationMenuLink {
    root: Node,
    title: Value<String>,
    description: Option<Value<String>>,
    icon: Option<View>,
    press: Option<Callback<()>>,
    variants: Variants,
}
/// Construct a navigation link, optionally inside a standard navigation menu panel.
pub fn navigation_menu_link(title: impl IntoText) -> NavigationMenuLink {
    let title = title.into_text_value();
    NavigationMenuLink {
        root: node().role(Role::Link).name(title),
        title,
        description: None,
        icon: None,
        press: None,
        variants: Variants::default(),
    }
}
impl NavigationMenuLink {
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(text.into_text_value());
        self
    }
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    pub fn on_press(mut self, press: impl Into<Callback<()>>) -> Self {
        self.press = Some(press.into());
        self
    }
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for NavigationMenuLink {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for NavigationMenuLink {
    fn into_view(mut self) -> View {
        let context = crate::navigation_context::current();
        if let Some(context) = &context {
            self.variants.merge(&context.variants);
        }
        let role = self.root.accessible_role().unwrap_or(Role::Link);
        let has_description = self.description.is_some();
        let behavior = kivact_interaction::use_button()
            .control()
            .map_state(move |input| NavigationLinkState {
                input,
                has_description,
            });
        let state = behavior.state();
        let variants = &self.variants;
        let title = with_recipe_style(
            text(self.title),
            |r| r.navigation_menu_recipes().link_title_recipe(),
            variants,
            move || state.get(),
        );
        let reference = node_ref();
        let description = self.description.map(|description| {
            with_recipe_style(
                text(description).node_ref(reference),
                |r| r.navigation_menu_recipes().link_description_recipe(),
                variants,
                move || state.get(),
            )
        });
        let mut root = self.root;
        if has_description {
            root = root.described_by(reference);
        }
        let icon = self.icon.map(|icon| {
            with_recipe_style(
                node().child(icon),
                |r| r.navigation_menu_recipes().link_icon_recipe(),
                variants,
                move || state.get(),
            )
        });
        let press = self.press;
        with_recipe_style(
            root.part(behavior).role(role).on_press(move || {
                if let Some(press) = press {
                    press.call(());
                }
                if let Some(context) = &context {
                    context.close.call(());
                }
            }),
            |r| r.navigation_menu_recipes().link_recipe(),
            variants,
            move || state.get(),
        )
        .child((icon, column((title, description))))
        .into_view()
    }
}
