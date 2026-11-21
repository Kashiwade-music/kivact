//! Role: Presentational checkbox, radio, and check marks for caller-owned semantics.
use crate::color as c;
use crate::metrics;
use crate::radius as r;

use kivact::component;
use kivact::custom::PartState;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::style;
use kivact::style::use_theme;
use kivact::ui::ControlSize;
use kivact::ui::theme::{CheckControl, CheckableState, CheckboxTheme, Look};
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

/// Square checkbox indicator, including the indeterminate state.
#[component]
pub fn checkbox_indicator(
    checked: Value<CheckedState>,
    #[prop(default = ControlSize::Medium)] size: Value<ControlSize>,
    #[prop(default = PartState::default())] input: PartState,
    content: Option<View>,
) -> impl Element {
    indicator(Kind::Checkbox, checked, size, input, content)
}

/// Circular radio indicator.
#[component]
pub fn radio_indicator(
    checked: Value<bool>,
    #[prop(default = ControlSize::Medium)] size: Value<ControlSize>,
    #[prop(default = PartState::default())] input: PartState,
    content: Option<View>,
) -> impl Element {
    indicator(
        Kind::Radio,
        (move || CheckedState::from_bool(checked.get())).into_value(),
        size,
        input,
        content,
    )
}

/// Standalone Lucide check. An unchecked check takes no space.
#[component]
pub fn check_indicator(
    checked: Value<bool>,
    #[prop(default = ControlSize::Medium)] size: Value<ControlSize>,
    #[prop(default = PartState::default())] input: PartState,
    content: Option<View>,
) -> impl Element {
    indicator(
        Kind::Check,
        (move || CheckedState::from_bool(checked.get())).into_value(),
        size,
        input,
        content,
    )
}

/// Build the root of an indicator of `kind`, marked by `content` or the built-in glyph.
fn indicator(
    kind: Kind,
    checked: Value<CheckedState>,
    size: Value<ControlSize>,
    input: PartState,
    children: Option<View>,
) -> Node {
    let size = size.get();
    let attachment = kivact::custom::part(|state| state);
    let actual = attachment.state();
    let root = node()
        .hit_testable(false)
        .with_node(move |node| node.part(attachment));
    let look = move || {
        let mut facts = input.get();
        if actual.get().is_disabled() && !facts.is_hidden() {
            facts.availability = kivact::view::Availability::Disabled;
        }
        if facts.is_disabled() {
            facts.focus = kivact::view::Focus::None;
        }
        Look::new(CheckableState {
            input: PartState::fixed(facts),
            control: match kind {
                Kind::Radio => CheckControl::Radio {
                    selected: checked.get().is_checked(),
                },
                Kind::Checkbox | Kind::Check => CheckControl::CheckBox(checked.get()),
            },
            size,
            invalid: false,
        })
    };
    match kind {
        Kind::Check => check_view(root, children, checked, look),
        kind => {
            let mark = children.unwrap_or_else(|| indicator_mark(kind, size, checked, look));
            root.with_node(move |node| {
                node.base_style(move || {
                    let look = look();
                    let theme = use_theme().get();
                    let mut base = theme
                        .ext::<CheckboxTheme>()
                        .indicator_recipe()
                        .style(&theme, &look)
                        .position(Position::Relative)
                        .left(0)
                        .right(0)
                        .top(0)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center);
                    if matches!(kind, Kind::Radio) {
                        base = base.border_radius(r::RADIUS_FULL);
                    }
                    if look.input.is_disabled() {
                        base = base
                            .opacity(metrics::DISABLED_OPACITY)
                            .border_color(c::BORDER);
                    }
                    base
                })
            })
            .child(mark)
        }
    }
}

fn check_view(
    root: Node,
    children: Option<View>,
    checked: Value<CheckedState>,
    look: impl Fn() -> Look<CheckableState> + Copy + 'static,
) -> Node {
    let custom = children.is_some();
    root.with_node(move |node| {
        node.base_style(move || {
            let state = look();
            let side = if custom || state.is_checked() { 16 } else { 0 };
            style()
                .size(side, side)
                .clip(true)
                .flex_shrink(0)
                .color(if state.input.is_disabled() {
                    c::ICON_DISABLED
                } else {
                    c::ACCENT
                })
        })
    })
    .child(children)
    .child((!custom).then(|| show(move || checked.get().is_checked(), || icon("check"))))
}

fn indicator_mark(
    kind: Kind,
    size: ControlSize,
    checked: Value<CheckedState>,
    look: impl Fn() -> Look<CheckableState> + Copy + 'static,
) -> View {
    match kind {
        Kind::Checkbox => (
            node().style(move || {
                let theme = use_theme().get();
                theme
                    .ext::<CheckboxTheme>()
                    .check_recipe()
                    .style(&theme, &look())
            }),
            node().style(move || {
                let theme = use_theme().get();
                theme
                    .ext::<CheckboxTheme>()
                    .mixed_recipe()
                    .style(&theme, &look())
            }),
        )
            .into_view(),
        Kind::Radio => show(
            move || checked.get().is_checked(),
            move || {
                node().style(move || {
                    let side = if matches!(size, ControlSize::Small | ControlSize::ExtraSmall) {
                        8
                    } else {
                        10
                    };
                    style()
                        .size(side, side)
                        .border_radius(r::RADIUS_FULL)
                        .background(c::ON_ACCENT)
                })
            },
        )
        .into_view(),
        Kind::Check => unreachable!("check handled separately"),
    }
}
