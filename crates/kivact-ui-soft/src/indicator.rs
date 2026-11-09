//! Role: Presentational checkbox, radio, and check marks for caller-owned semantics.

use crate::recipes_controls_choice::{
    checkbox_check, checkbox_indicator as checkbox_surface, checkbox_mixed,
};

use kivact::custom::PartState;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::style;
use kivact::style::use_theme;
use kivact::ui::CheckableState;
use kivact::ui::ControlSize;
use kivact::ui::Look;
use kivact::view::CheckedState;
use kivact::view::Element;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::View;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::show;

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
    size: Value<ControlSize>,
    children: Option<View>,
}

fn indicator(kind: Kind, checked: Value<CheckedState>) -> Indicator {
    Indicator {
        root: node().hit_testable(false),
        kind,
        checked,
        input: PartState::default.into_value(),
        size: (ControlSize::Medium).into_value(),
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
    /// Choose the indicator size; medium by default.
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }

    /// Follow the owning control's actual hover, press, focus, and disabled state.
    pub fn input(mut self, state: impl IntoValue<PartState>) -> Self {
        self.input = state.into_value();
        self
    }

    /// Mount a custom mark once instead of the built-in check or radio glyph.
    pub fn content(mut self, content: impl IntoView) -> Self {
        self.children = Some(content.into_view());
        self
    }
}

impl Element for Indicator {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Indicator {
    fn into_view(mut self) -> View {
        let size = self.size.get();
        let attachment = kivact::custom::part(|state| state);
        let actual = attachment.state();
        self.root = self.root.with_node(move |node| node.part(attachment));
        let checked = self.checked;
        let input = self.input;
        let look = move || {
            let mut input = input.get();
            input.disabled |= actual.get().disabled;
            input.focus_visible &= !input.disabled;
            Look::new(CheckableState {
                input,
                checked: checked.get(),
                size,
                invalid: false,
                text: None,
                switch_drag: None,
            })
        };
        match self.kind {
            Kind::Check => check_view(self.root, self.children, checked, look),
            kind => {
                let mark = self
                    .children
                    .unwrap_or_else(|| indicator_mark(kind, size, checked, look));
                self.root
                    .with_node(move |node| {
                        node.base_style(move || {
                            let t = kivact::style::use_tokens::<crate::Tokens>();
                            let look = look();
                            let mut base = checkbox_surface(&use_theme().get(), &look)
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
                    })
                    .child(mark)
                    .into_view()
            }
        }
    }
}

fn check_view(
    root: Node,
    children: Option<View>,
    checked: Value<CheckedState>,
    look: impl Fn() -> Look<CheckableState> + Copy + 'static,
) -> View {
    let custom = children.is_some();
    root.with_node(move |node| {
        node.base_style(move || {
            let t = kivact::style::use_tokens::<crate::Tokens>();
            let state = look();
            let side = if custom || state.checked.is_checked() {
                16
            } else {
                0
            };
            style()
                .size(side, side)
                .clip(true)
                .shrink(0)
                .color(if state.input.disabled {
                    t.colors.icon_disabled
                } else {
                    t.colors.accent
                })
        })
    })
    .child(children)
    .child((!custom).then(|| show(move || checked.get().is_checked(), || icon("check"))))
    .into_view()
}

fn indicator_mark(
    kind: Kind,
    size: ControlSize,
    checked: Value<CheckedState>,
    look: impl Fn() -> Look<CheckableState> + Copy + 'static,
) -> View {
    match kind {
        Kind::Checkbox => (
            node().style(move || checkbox_check(&use_theme().get(), &look())),
            node().style(move || checkbox_mixed(&use_theme().get(), &look())),
        )
            .into_view(),
        Kind::Radio => show(
            move || checked.get().is_checked(),
            move || {
                node().style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let side = if matches!(size, ControlSize::Small | ControlSize::ExtraSmall) {
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
    }
}
