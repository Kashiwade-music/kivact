//! Role: The spinner builder and its animated rendering.

use crate::ControlSize;
use crate::spinner::look::SpinnerTheme;
use crate::theme::SpinnerState;
use kivact_core::motion::{Animated, Easing, Repeat, Transition};
use kivact_core::state::{IntoValue, Value};
use kivact_core::view::{Element, IntoView, Node, Role, View, node};
pub struct Spinner {
    root: Node,
    size: Value<ControlSize>,
    label: Option<View>,
}
#[track_caller]
pub fn spinner() -> Spinner {
    Spinner {
        root: node()
            .role(Role::Status)
            .name(kivact_core::i18n::message("spinner.loading", "Loading")),
        size: crate::control_size::default_control_size().into_value(),
        label: None,
    }
}
impl Spinner {
    #[track_caller]
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
    pub fn label_content(mut self, label: impl IntoView) -> Self {
        self.label = Some(label.into_view());
        self
    }
}
impl Element for Spinner {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Spinner {
    fn into_view(self) -> View {
        let variants = self.root.authored().variants();
        let theme = kivact_core::style::use_theme().get();
        let period_ms = theme.get(&crate::spinner::look::PERIOD_MS_VAR);
        let reduced_period = *theme
            .ext::<crate::spinner::look::SpinnerTheme>()
            .reduced_period_value();
        let size = self.size;
        let variants = variants;
        let glyph = kivact_core::view::dynamic(
            move || {
                let reduced = kivact_core::platform::is_motion_reduced().get();
                let essential = reduced && reduced_period.is_some();
                let period = if reduced {
                    reduced_period.unwrap_or(period_ms)
                } else {
                    period_ms
                };
                (period, essential)
            },
            move |(period, essential)| {
                let transition = Transition::tween(period, Easing::Linear).repeat(Repeat::Forever);
                let transition = if essential {
                    transition.essential()
                } else {
                    transition
                };
                let turn = Animated::from(0.0, 360.0, transition);
                crate::layer::with_base_recipe_style(
                    node(),
                    |r: &SpinnerTheme| r.root_recipe(),
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
                        .flex_direction(kivact_core::style::FlexDirection::Column)
                        .align_items(kivact_core::style::Align::Center)
                        .gap(crate::spinner::look::GAP)
                        .flex_shrink(0)
                })
            })
            .child(glyph)
            .child(self.label)
            .into_view()
    }
}
