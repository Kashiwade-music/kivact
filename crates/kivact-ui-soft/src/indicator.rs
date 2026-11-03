//! Role: Presentational checkbox, radio, and check marks for caller-owned semantics.

use crate::recipes_controls_choice::{
    checkbox_check, checkbox_indicator as checkbox_surface, checkbox_mixed,
};
use crate::tokens;
use kivact::Value;
use kivact::control::{Node, NodeBuilder, icon, node};
use kivact::primitive::{NodeExt, PartState};
use kivact::style::{Align, Justify, Position, current_theme, style};
use kivact::ui::{CheckState, ControlSize, Look};
use kivact::view::{CheckedState, IntoValue, IntoView, View, show};

#[derive(Clone, Copy)]
enum Kind {
    Checkbox,
    Radio,
    Check,
}

/// A purely visual selection indicator.
pub struct Indicator {
    root: Node,
    kind: Kind,
    checked: Value<CheckedState>,
    input: Value<PartState>,
    size: ControlSize,
    disabled: Option<Value<bool>>,
    children: Option<View>,
}

fn indicator(kind: Kind, checked: Value<CheckedState>) -> Indicator {
    Indicator {
        root: node().hit_testable(false),
        kind,
        checked,
        input: PartState::default.into_value(),
        size: ControlSize::Medium,
        disabled: None,
        children: None,
    }
}

/// Square checkbox indicator, including the indeterminate state.
#[track_caller]
pub fn checkbox_indicator(checked: impl IntoValue<CheckedState>) -> Indicator {
    indicator(Kind::Checkbox, checked.into_value())
}

/// Circular radio indicator.
#[track_caller]
pub fn radio_indicator(checked: impl IntoValue<bool>) -> Indicator {
    let checked = checked.into_value();
    indicator(
        Kind::Radio,
        (move || CheckedState::from_bool(checked.get())).into_value(),
    )
}

/// Standalone Lucide check. An unchecked check takes no space.
#[track_caller]
pub fn check_indicator(checked: impl IntoValue<bool>) -> Indicator {
    let checked = checked.into_value();
    indicator(
        Kind::Check,
        (move || CheckedState::from_bool(checked.get())).into_value(),
    )
}

impl Indicator {
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Follow the owning control's actual hover, press, focus, and disabled state.
    pub fn input(mut self, state: impl IntoValue<PartState>) -> Self {
        self.input = state.into_value();
        self
    }

    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = Some(disabled.into_value());
        self
    }

    pub fn content(mut self, content: impl IntoView) -> Self {
        self.children = Some(content.into_view());
        self
    }
}

impl NodeBuilder for Indicator {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Indicator {
    fn into_view(self) -> View {
        let size = self.size;
        let disabled = self.disabled;
        let checked = self.checked;
        let input = self.input;
        let look = move || {
            let mut input = input.get();
            input.disabled |= disabled.is_some_and(|disabled| disabled.get());
            input.focus_visible &= !input.disabled;
            Look::new(CheckState {
                input,
                checked: checked.get(),
                size,
                invalid: false,
                text: None,
            })
        };
        match self.kind {
            Kind::Check => {
                let custom = self.children.is_some();
                self.root
                    .base_style(move || {
                        let t = tokens();
                        let state = look();
                        let side = if custom || state.checked.is_checked() {
                            16
                        } else {
                            0
                        };
                        style().size(side, side).clip(true).shrink(0).color(
                            if state.input.disabled {
                                t.colors.icon_disabled
                            } else {
                                t.colors.accent
                            },
                        )
                    })
                    .child(self.children)
                    .child(
                        (!custom)
                            .then(|| show(move || checked.get().is_checked(), || icon("check"))),
                    )
                    .into_view()
            }
            kind => {
                let mark = self.children.unwrap_or_else(|| match kind {
                    Kind::Checkbox => (
                        node().style(move || checkbox_check(&current_theme(), &look())),
                        node().style(move || checkbox_mixed(&current_theme(), &look())),
                    )
                        .into_view(),
                    Kind::Radio => show(
                        move || checked.get().is_checked(),
                        move || {
                            node().style(move || {
                                let t = tokens();
                                let side =
                                    if matches!(size, ControlSize::Small | ControlSize::ExtraSmall)
                                    {
                                        8
                                    } else {
                                        10
                                    };
                                style()
                                    .size(side, side)
                                    .radius(t.radius.full)
                                    .background(t.colors.on_accent)
                            })
                        },
                    )
                    .into_view(),
                    Kind::Check => unreachable!("check handled separately"),
                });
                self.root
                    .base_style(move || {
                        let t = tokens();
                        let look = look();
                        let mut base = checkbox_surface(&current_theme(), &look)
                            .position(Position::Relative)
                            .left(0)
                            .right(0)
                            .top(0)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center);
                        if matches!(kind, Kind::Radio) {
                            base = base.radius(t.radius.full);
                        }
                        if look.input.disabled {
                            base = base
                                .opacity(t.sizes.disabled_opacity)
                                .border_color(t.colors.border);
                        }
                        base
                    })
                    .child(mark)
                    .into_view()
            }
        }
    }
}
