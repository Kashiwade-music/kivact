//! Role: Render rich navigation links using the closest existing panel's close request.
use crate::layer::with_recipe_style;
use crate::navigation_menu::look::NavigationLinkState;
use crate::navigation_menu::look::NavigationMenuTheme;
use crate::variants::{Variant, Variants};
use kivact_core::UiError;
use kivact_core::state::{IntoText, Value};
use kivact_core::view::{Element, IntoView, Node, Role, View, column, node, node_ref, text};
/// A title, optional icon and description that activates and closes its navigation panel.
pub struct NavigationMenuLink {
    root: Node,
    title: Value<String>,
    icon: Option<View>,
    variants: Variants,
}
/// Construct a navigation link, optionally inside a standard navigation menu panel.
#[track_caller]
pub fn navigation_menu_link(title: impl IntoText) -> NavigationMenuLink {
    let title = title.into_text_value();
    NavigationMenuLink {
        root: node().role(Role::Link).name(title),
        title,
        icon: None,
        variants: Variants::default(),
    }
}
impl NavigationMenuLink {
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    pub fn variant(mut self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl Element for NavigationMenuLink {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for NavigationMenuLink {
    fn into_view(mut self) -> View {
        let context = crate::navigation_menu::context::current();
        if let Some(context) = &context {
            self.variants = self.variants.with_defaults(&context.variants);
        }
        let role = self.root.accessible_role().unwrap_or(Role::Link);
        let description = self.root.authored().description();
        let has_description = description.is_some();
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
            |r: &NavigationMenuTheme| r.link_title_recipe(),
            variants,
            move || state.get(),
        );
        let reference = node_ref();
        let description = description.map(|description| {
            with_recipe_style(
                text(description).node_ref(reference),
                |r: &NavigationMenuTheme| r.link_description_recipe(),
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
                |r: &NavigationMenuTheme| r.link_icon_recipe(),
                variants,
                move || state.get(),
            )
        });
        let caller = root.take_press();
        with_recipe_style(
            root.part(behavior)
                .role(role)
                .on_press(move || -> Result<(), UiError> {
                    if let Some(caller) = &caller {
                        caller.call(None)?;
                    }
                    if let Some(context) = &context {
                        context.close.call(());
                    }
                    Ok(())
                }),
            |r: &NavigationMenuTheme| r.link_recipe(),
            variants,
            move || state.get(),
        )
        .child((icon, column((title, description))))
        .into_view()
    }
}
