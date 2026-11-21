//! Role: The Badge builder and its icon, label and activation rendering.

use crate::badge::look::BadgeTheme;
use crate::theme::{BadgeState, Variant};
use crate::variants::Variants;
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::view::{Element, IntoView, Node, View, row, text};
use kivact_interaction::Tone;
// xtask-allow(state): legacy: icon, trailing
pub struct Badge {
    root: Node,
    label: Value<String>,
    tone: Value<Tone>,
    variants: Variants,
    icon: Option<View>,
    trailing: Option<View>,
}
#[track_caller]
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
    #[track_caller]
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
}
impl Element for Badge {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Badge {
    fn into_view(self) -> View {
        let tone = self.tone;
        let actionable = self.root.authored().has_press();
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
        if actionable {
            root = root.part(kivact_interaction::use_button().control());
        }
        crate::layer::with_base_recipe_style(
            root,
            |r: &BadgeTheme| r.root_recipe(),
            &self.variants,
            move || state.get(),
        )
        .child(self.icon)
        .child(crate::layer::with_recipe_style(
            text(self.label),
            |r: &BadgeTheme| r.label_recipe(),
            &self.variants,
            move || state.get(),
        ))
        .child(self.trailing)
        .into_view()
    }
}
