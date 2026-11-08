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
    pub fn variant(mut self, variant: impl Variant) -> Self {
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
        let metrics = kivact_core::style::use_tokens::<crate::UiTokens>()
            .indicators
            .clone();
        let size = self.size;
        let variants = self.variants;
        let glyph = kivact_core::view::dynamic(
            move || {
                let reduced = kivact_core::platform::is_motion_reduced().get();
                let essential = reduced && metrics.reduced_period_ms.is_some();
                let period = if reduced {
                    metrics.reduced_period_ms.unwrap_or(metrics.period_ms)
                } else {
                    metrics.period_ms
                };
                (period, essential)
            },
            move |(period, essential)| {
                let transition =
                    Transition::tween(period as f64, Easing::Linear).repeat(Repeat::Forever);
                let transition = if essential {
                    transition.essential()
                } else {
                    transition
                };
                let turn = Animated::from(0.0, 360.0, transition);
                crate::layer::with_base_recipe_style(
                    node(),
                    |r| r.spinner_recipe(),
                    &variants,
                    move || SpinnerState {
                        size: size.get(),
                        turn: turn.get(),
                    },
                )
            },
        );
        self.root
            .with_node(|node| {
                node.base_style(move || {
                    kivact_core::style::style()
                        .direction(kivact_core::style::Direction::Column)
                        .align_items(kivact_core::style::Align::Center)
                        .gap(
                            kivact_core::style::use_tokens::<crate::UiTokens>()
                                .indicators
                                .gap,
                        )
                        .shrink(0)
                })
            })
            .child(glyph)
            .child(self.label)
            .into_view()
    }
}
