//! Role: Compose a semantic badge with icons and optional activation.
use crate::variants::Variants;
use crate::{BadgeState, Variant};
use kivact_core::state::{Callback, IntoText, IntoValue, Value};
use kivact_core::view::{IntoView, Node, NodeBuilder, View, row, text};
use kivact_interaction::Tone;
pub struct Badge {
    root: Node,
    label: Value<String>,
    tone: Value<Tone>,
    variants: Variants,
    // xtask-allow(state-fields): builder option - icon
    icon: Option<View>,
    // xtask-allow(state-fields): builder option - trailing
    trailing: Option<View>,
    on_press: Option<Callback<()>>,
}
pub fn badge(label: impl IntoText) -> Badge {
    let label = label.into_text_value();
    Badge {
        root: row(()).name(label),
        label,
        tone: Tone::Neutral.into_value(),
        variants: Variants::default(),
        icon: None,
        trailing: None,
        on_press: None,
    }
}
impl Badge {
    pub fn tone(mut self, tone: impl IntoValue<Tone>) -> Self {
        self.tone = tone.into_value();
        self
    }
    pub fn variant(mut self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    pub fn trailing_icon(mut self, icon: impl IntoView) -> Self {
        self.trailing = Some(icon.into_view());
        self
    }
    pub fn on_press(mut self, f: impl Into<Callback<()>>) -> Self {
        self.on_press = Some(f.into());
        self
    }
}
impl NodeBuilder for Badge {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Badge {
    fn into_view(self) -> View {
        let tone = self.tone;
        let actionable = self.on_press.is_some();
        let leading_icon = self.icon.is_some();
        let trailing_icon = self.trailing.is_some();
        let part = kivact_core::custom::part(move |input| BadgeState {
            tone: tone.get(),
            input,
            actionable,
            leading_icon,
            trailing_icon,
        });
        let state = part.state();
        let mut root = self.root.part(part);
        if let Some(on_press) = self.on_press {
            root = root
                .part(kivact_interaction::use_button().control())
                .on_press(move || on_press.call(()));
        }
        crate::layer::with_base_recipe_style(
            root,
            |r| r.badge_recipe(),
            &self.variants,
            move || state.get(),
        )
        .child(self.icon)
        .child(crate::layer::with_recipe_style(
            text(self.label),
            |r| r.badge_label_recipe(),
            &self.variants,
            move || state.get(),
        ))
        .child(self.trailing)
        .into_view()
    }
}
