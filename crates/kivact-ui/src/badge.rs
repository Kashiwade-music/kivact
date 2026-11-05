//! Role: Compose a semantic badge with icons and optional activation.
use crate::variants::Variants;
use crate::{BadgeState, Variant};
use kivact_core::state::{Callback, IntoText, IntoValue, Value};
use kivact_core::view::{IntoView, Node, NodeBuilder, View, row, text};
use kivact_interaction::Tone;
// xtask-allow(state-fields): Builder inputs and optional content are independent configuration; image state belongs to NodeRef.
pub struct Badge {
    root: Node,
    label: Value<String>,
    tone: Value<Tone>,
    variants: Variants,
    icon: Option<View>,
    trailing: Option<View>,
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
    }
}
impl Badge {
    pub fn tone(mut self, tone: impl IntoValue<Tone>) -> Self {
        self.tone = tone.into_value();
        self
    }
    pub fn variant(self, variant: impl Variant) -> Self {
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
        let f = f.into();
        self.root = self
            .root
            .part(kivact_interaction::use_button().control())
            .on_press(move || f.call(()));
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
        crate::layer::with_base_recipe_style(
            self.root,
            |r| r.badge_recipe(),
            &self.variants,
            move || BadgeState { tone: tone.get() },
        )
        .child(self.icon)
        .child(crate::layer::with_recipe_style(
            text(self.label),
            |r| r.badge_label_recipe(),
            &self.variants,
            move || BadgeState { tone: tone.get() },
        ))
        .child(self.trailing)
        .into_view()
    }
}
