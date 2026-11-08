//! Role: Build accessible empty messages with opt-in announcements.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use kivact_core::state::{IntoText, IntoValue, Value, on_mount};
use kivact_core::view::{IntoView, LiveRegion, Node, NodeBuilder, Role, View, node, text};
use kivact_interaction::announcer;

#[derive(Clone, Copy, PartialEq)]
enum AnnouncePolicy {
    Silent,
    Once,
    Changes,
}

use crate::component_values::EmptyStateState;

// xtask-allow(state-fields): Builder configuration and optional slots; the one-shot announcement runs in on_mount.
/// A message for a region without content.
pub struct EmptyState {
    root: Node,
    title: Value<String>,
    // xtask-allow(state-fields): builder option - description
    description: Option<Value<String>>,
    // xtask-allow(state-fields): builder option - icon
    icon: Option<View>,
    // xtask-allow(state-fields): builder option - actions
    actions: Option<View>,
    heading_level: Value<u8>,
    announce: AnnouncePolicy,
    variants: Variants,
}
/// Describe an empty region with a visible heading.
pub fn empty_state(title: impl IntoText) -> EmptyState {
    EmptyState {
        root: node(),
        title: title.into_text_value(),
        description: None,
        icon: None,
        actions: None,
        heading_level: 3.into_value(),
        announce: AnnouncePolicy::Silent,
        variants: Variants::default(),
    }
}
impl EmptyState {
    /// Set supporting text.
    pub fn description(mut self, value: impl IntoText) -> Self {
        self.description = Some(value.into_text_value());
        self
    }
    /// Set the decorative icon or illustration.
    pub fn icon(mut self, value: impl IntoView) -> Self {
        self.icon = Some(value.into_view());
        self
    }
    /// Set actions below the description.
    pub fn actions(mut self, value: impl IntoView) -> Self {
        self.actions = Some(value.into_view());
        self
    }
    /// Set the heading level, clamped to 1 through 6.
    pub fn heading_level(mut self, level: impl IntoValue<u8>) -> Self {
        self.heading_level = level.into_value();
        self
    }
    /// Announce the message politely once after mounting.
    pub fn announce(mut self) -> Self {
        self.announce = AnnouncePolicy::Once;
        self
    }
    /// Announce normalized message changes and late visibility, skipping the initial render.
    /// Actual live-region ancestors suppress duplicate announcements.
    pub fn announce_changes(mut self) -> Self {
        self.announce = AnnouncePolicy::Changes;
        self
    }
    /// Add a theme-defined appearance.
    pub fn variant(mut self, value: impl Variant) -> Self {
        self.variants.add(value);
        self
    }
}
impl NodeBuilder for EmptyState {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for EmptyState {
    fn into_view(self) -> View {
        let heading_level = self.heading_level;
        let state = move || EmptyStateState {
            heading_level: heading_level.get().clamp(1, 6),
        };
        let title = self.title;
        let description = self.description;
        let notices = (self.announce == AnnouncePolicy::Once).then(|| {
            let notices = announcer();
            on_mount(move || {
                let message = description.map_or_else(
                    || title.get(),
                    |description| format!("{}. {}", title.get(), description.get()),
                );
                notices.announce(message, LiveRegion::Polite);
            });
            notices.outlet()
        });
        let variants = &self.variants;
        let icon = self.icon.map(|icon| {
            with_recipe_style(
                node().child(icon),
                |r| r.empty_state_recipes().icon_recipe(),
                variants,
                state,
            )
        });
        let title = with_recipe_style(
            text(title)
                .role(Role::Heading)
                .level(move || u32::from(state().heading_level)),
            |r| r.empty_state_recipes().title_recipe(),
            variants,
            state,
        );
        let description = description.map(|value| {
            with_recipe_style(
                text(value),
                |r| r.empty_state_recipes().description_recipe(),
                variants,
                state,
            )
        });
        let actions = self.actions.map(|view| {
            with_recipe_style(
                node().child(view),
                |r| r.empty_state_recipes().actions_recipe(),
                variants,
                state,
            )
        });
        let result = with_base_recipe_style(
            self.root,
            |r| r.empty_state_recipes().root_recipe(),
            variants,
            state,
        )
        .child((icon, title, description, actions, notices));
        if self.announce == AnnouncePolicy::Changes {
            crate::empty_state_changes::observe(self.title, self.description, result)
        } else {
            result.into_view()
        }
    }
}
