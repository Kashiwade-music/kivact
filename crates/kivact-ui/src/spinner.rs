//! Role: Display a localized loading status with one animation.
use crate::variants::{Variant, Variants};
use crate::{ControlSize, SpinnerState};
use kivact_core::motion::{Animated, Easing, Repeat, Transition};
use kivact_core::state::{IntoValue, Value};
use kivact_core::view::{IntoView, Node, NodeBuilder, Role, View, node};
// xtask-allow(state-fields): Builder inputs and optional content are independent configuration; image state belongs to NodeRef.
pub struct Spinner {
    root: Node,
    size: Value<ControlSize>,
    variants: Variants,
    label: Option<View>,
}
pub fn spinner() -> Spinner {
    Spinner {
        root: node()
            .role(Role::Status)
            .name(kivact_core::i18n::message("spinner.loading", "Loading")),
        size: crate::values::default_control_size().into_value(),
        variants: Variants::default(),
        label: None,
    }
}
impl Spinner {
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
    pub fn label_content(mut self, label: impl IntoView) -> Self {
        self.label = Some(label.into_view());
        self
    }
}
impl NodeBuilder for Spinner {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Spinner {
    fn into_view(self) -> View {
        let period = kivact_core::style::use_tokens::<crate::UiTokens>()
            .indicators
            .period_ms;
        let turn = Animated::from(
            0.0,
            360.0,
            Transition::tween(period as f64, Easing::Linear).repeat(Repeat::Forever),
        );
        let size = self.size;
        let glyph = crate::layer::with_base_recipe_style(
            node(),
            |r| r.spinner_recipe(),
            &self.variants,
            move || SpinnerState {
                size: size.get(),
                turn: turn.get(),
            },
        );
        self.root.child(glyph).child(self.label).into_view()
    }
}
