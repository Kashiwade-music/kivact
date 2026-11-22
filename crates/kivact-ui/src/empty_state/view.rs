//! Role: The empty state component: an icon, a message and actions.

use super::changes;
use crate::empty_state::look::{EmptyStateState, EmptyStateTheme};
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::custom::Caller;
use kivact_core::state::{Value, on_mount};
use kivact_core::view::{LiveRegion, Role, View, node, text};
use kivact_interaction::use_announcer;
use kivact_macros::component;

/// A message for a region without content: its title, the caller's
/// description and an optional icon and actions.
///
/// `announce(true)` announces the message politely once after mounting.
/// `announce_changes(true)` announces normalized message changes and late
/// visibility, skipping the initial render, and wins over `announce`. Actual
/// live-region ancestors suppress duplicate announcements.
#[component]
pub fn empty_state(
    title: Value<String>,
    icon: Option<View>,
    actions: Option<View>,
    #[prop(default = 3u8)] heading_level: Value<u8>,
    #[prop(default = false)] announce: Value<bool>,
    #[prop(default = false)] announce_changes: Value<bool>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let state = move || EmptyStateState {
        heading_level: heading_level.get().clamp(1, 6),
    };
    let description = caller.description();
    let changes = announce_changes.get_untracked();
    let once = !changes && announce.get_untracked();
    let outlet = once.then(|| {
        let notices = use_announcer();
        on_mount(move || {
            let message = description.map_or_else(
                || title.get(),
                |description| format!("{}. {}", title.get(), description.get()),
            );
            notices.announce(message, LiveRegion::Polite);
        });
        crate::announcer_outlet(notices)
    });
    let icon = icon.map(|view| {
        with_recipe_style(
            node().child(view),
            |r: &EmptyStateTheme| r.icon_recipe(),
            &variants,
            state,
        )
    });
    let title_view = with_recipe_style(
        text(title)
            .role(Role::Heading)
            .level(move || u32::from(state().heading_level)),
        |r: &EmptyStateTheme| r.title_recipe(),
        &variants,
        state,
    );
    let description_view = description.map(|value| {
        with_recipe_style(
            text(value),
            |r: &EmptyStateTheme| r.description_recipe(),
            &variants,
            state,
        )
    });
    let actions = actions.map(|view| {
        with_recipe_style(
            node().child(view),
            |r: &EmptyStateTheme| r.actions_recipe(),
            &variants,
            state,
        )
    });
    let result = with_base_recipe_style(
        node(),
        |r: &EmptyStateTheme| r.root_recipe(),
        &variants,
        state,
    )
    .child((icon, title_view, description_view, actions, outlet));
    if changes {
        changes::observe(title, description, result)
    } else {
        result
    }
}
