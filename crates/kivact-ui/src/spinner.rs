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
        let metrics = kivact_core::style::use_tokens::<crate::UiTokens>()
            .indicators
            .clone();
        let turn = match metrics.reduced_period_ms {
            Some(reduced_period) => {
                let elapsed = kivact_core::state::signal(0.0f64);
                kivact_core::motion::frame_loop(move |frame| {
                    elapsed.set(frame.now.as_secs_f64() * 1000.0);
                })
                .start(kivact_core::motion::FrameRate::Display);
                kivact_core::state::memo(move || {
                    let period = if kivact_core::platform::is_motion_reduced().get() {
                        reduced_period
                    } else {
                        metrics.period_ms
                    }
                    .max(1) as f64;
                    elapsed.get().rem_euclid(period) * 360.0 / period
                })
            }
            None => Animated::from(
                0.0,
                360.0,
                Transition::tween(metrics.period_ms as f64, Easing::Linear).repeat(Repeat::Forever),
            )
            .value(),
        };
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
