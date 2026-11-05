//! Role: Present persistent alerts and dismissible feedback banners.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use kivact_core::custom::Owner;
use kivact_core::state::{
    Binding, Callback, IntoBinding, IntoText, IntoValue, Value, effect, signal,
};
use kivact_core::view::{
    IntoView, IntoViewFn, LiveRegion, Node, NodeBuilder, Role, View, ViewFn, node, node_ref, show,
    text,
};
use kivact_interaction::Tone;

/// The structural presentation of an alert.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertVariant {
    #[default]
    Default,
    Banner,
}
impl Variant for AlertVariant {}
#[derive(Clone, Copy, PartialEq)]
enum BannerState {
    Shown,
    Dismissed,
}
use crate::component_values::AlertState;

// xtask-allow(state-fields): Independent builder slots and inputs; persistent dismissal uses BannerState.
/// An urgent, persistent message with optional actions and details.
pub struct Alert {
    root: Node,
    title: Value<String>,
    description: Option<Value<String>>,
    icon: Option<View>,
    actions: Option<View>,
    content: Option<ViewFn>,
    collapsible: Value<bool>,
    open: Option<Binding<bool>>,
    tone: Value<Tone>,
    dismissible: Value<bool>,
    dismiss_label: Value<String>,
    dismiss: Option<Callback<()>>,
    variants: Variants,
}
/// Display an alert without moving keyboard focus or automatically hiding it.
pub fn alert(title: impl IntoText) -> Alert {
    Alert {
        root: node(),
        title: title.into_text_value(),
        description: None,
        icon: None,
        actions: None,
        content: None,
        collapsible: false.into_value(),
        open: None,
        tone: Tone::Neutral.into_value(),
        dismissible: false.into_value(),
        dismiss_label: "Dismiss notification".into_text_value(),
        dismiss: None,
        variants: Variants::default(),
    }
}
impl Alert {
    /// Set supporting text.
    pub fn description(mut self, value: impl IntoText) -> Self {
        self.description = Some(value.into_text_value());
        self
    }
    /// Set the semantic tone.
    pub fn tone(mut self, tone: impl IntoValue<Tone>) -> Self {
        self.tone = tone.into_value();
        self
    }
    /// Set the decorative icon.
    pub fn icon(mut self, value: impl IntoView) -> Self {
        self.icon = Some(value.into_view());
        self
    }
    /// Set trailing actions.
    pub fn actions(mut self, value: impl IntoView) -> Self {
        self.actions = Some(value.into_view());
        self
    }
    /// Build details each time they are expanded.
    pub fn content<V: IntoView>(mut self, content: impl IntoViewFn<V>) -> Self {
        self.content = Some(content.into_view_fn());
        self
    }
    /// Make details collapsible.
    pub fn collapsible(mut self, value: impl IntoValue<bool>) -> Self {
        self.collapsible = value.into_value();
        self
    }
    /// Bind expanded details to application state.
    pub fn open(mut self, value: impl IntoBinding<bool>) -> Self {
        self.open = Some(value.into_binding());
        self
    }
    /// Show a dismiss control; dismissal lasts until remount.
    pub fn dismissible(mut self, value: impl IntoValue<bool>) -> Self {
        self.dismissible = value.into_value();
        self
    }
    /// Name the dismiss control.
    pub fn dismiss_label(mut self, label: impl IntoText) -> Self {
        self.dismiss_label = label.into_text_value();
        self
    }
    /// Show a dismiss control and report dismissal after restoring focus.
    pub fn on_dismiss(mut self, f: impl Into<Callback<()>>) -> Self {
        self.dismissible = true.into_value();
        self.dismiss = Some(f.into());
        self
    }
    /// Set a structural or theme-defined appearance.
    pub fn variant(self, value: impl Variant) -> Self {
        self.variants.add(value);
        self
    }
}
impl NodeBuilder for Alert {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Alert {
    fn into_view(self) -> View {
        let phase = signal(BannerState::Shown);
        let reference = node_ref();
        let within = reference.focus_within();
        let owner = Owner::current();
        let previous = signal(None);
        effect(move || {
            let focused = owner.focused_node();
            if phase.get() == BannerState::Shown && !within.get() && focused.is_some() {
                previous.set(focused);
            }
        });
        let tone = self.tone;
        let dismissible = self.dismissible;
        let collapsible = self.collapsible;
        let has_details = self.content.is_some();
        let open = self.open.unwrap_or_else(|| signal(false).into_binding());
        let state = move || AlertState {
            tone: tone.get(),
            expanded: (has_details && collapsible.get()).then(|| open.get()),
            dismissible: dismissible.get(),
        };
        let variants = &self.variants;
        let icon = self.icon.map(|view| {
            with_recipe_style(
                node().child(view),
                |r| r.alert_recipes().icon_recipe(),
                variants,
                state,
            )
        });
        let title = with_recipe_style(
            text(self.title),
            |r| r.alert_recipes().title_recipe(),
            variants,
            state,
        );
        let description = self.description.map(|value| {
            with_recipe_style(
                text(value),
                |r| r.alert_recipes().description_recipe(),
                variants,
                state,
            )
        });
        let actions = self.actions.map(|view| {
            with_recipe_style(
                node().child(view),
                |r| r.alert_recipes().actions_recipe(),
                variants,
                state,
            )
        });
        let callback = self.dismiss;
        let dismiss = super::button(self.dismiss_label)
            .on_press(move || {
                if phase.get_untracked() == BannerState::Dismissed {
                    return;
                }
                phase.set(BannerState::Dismissed);
                if let Some(previous) = previous.get_untracked() {
                    previous.restore_focus();
                }
                if let Some(callback) = callback {
                    callback.call(());
                }
            })
            .hidden(move || !dismissible.get());
        let toggle = has_details.then(|| {
            super::button(move || {
                if open.get() {
                    "Collapse details"
                } else {
                    "Expand details"
                }
            })
            .with_node(move |node| node.expanded(move || open.get()))
            .hidden(move || !collapsible.get())
            .on_press(move || open.set(!open.get()))
        });
        let content = self.content.map(|content| {
            let variants = self.variants.clone();
            show(
                move || !collapsible.get() || open.get(),
                move || {
                    with_recipe_style(
                        node().child(content.call(())),
                        |r| r.alert_recipes().content_recipe(),
                        &variants,
                        state,
                    )
                },
            )
        });
        with_base_recipe_style(
            self.root,
            |r| r.alert_recipes().root_recipe(),
            variants,
            state,
        )
        .role(Role::Alert)
        .name(self.title)
        .live_region(LiveRegion::Assertive)
        .node_ref(reference)
        .hidden(move || phase.get() == BannerState::Dismissed)
        .child((icon, title, description, actions, toggle, content, dismiss))
        .into_view()
    }
}
