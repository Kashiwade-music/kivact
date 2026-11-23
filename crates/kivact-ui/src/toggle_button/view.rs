//! Role: The toggle button component: a button with a pressed state.

use crate::control_size::{ControlSize, default_control_size};
use crate::layer::with_base_recipe_style;
use crate::toggle_button::look::{ToggleButtonState, ToggleButtonTheme, ToggleLabelReservation};
use kivact_core::custom::{Caller, Constraints, primitive};
use kivact_core::draw::TextFormat;
use kivact_core::geometry::Size;
use kivact_core::state::{Callback, IntoValue, Signal, Value};
use kivact_core::style::{Align, TextAlign, style, use_theme};
use kivact_core::view::{
    EditPhase, Element, IntoView, Node, View, column, dynamic, node, show, text,
};
use kivact_interaction::{ToggleState, use_toggle_button};
use kivact_macros::component;

fn toggle_label(
    label: Value<String>,
    hidden: Option<Value<bool>>,
    size: Value<ControlSize>,
) -> View {
    dynamic(
        move || {
            (
                *use_theme()
                    .get()
                    .ext::<ToggleButtonTheme>()
                    .label_reservation_value(),
                size.get(),
            )
        },
        move |(reservation, size)| {
            let ToggleLabelReservation::Reserve {
                weight,
                size: font_size,
                extra_small_size,
            } = reservation
            else {
                return crate::helpers::visible_label(label, hidden);
            };
            let font_size = if size == ControlSize::ExtraSmall {
                extra_small_size
            } else {
                font_size
            };
            let visible = move || {
                let reservation = primitive()
                    .measure(
                        label,
                        move |label: &String, constraints: Constraints<'_>| {
                            let text = TextFormat {
                                weight,
                                size: font_size,
                                ..TextFormat::default()
                            };
                            let width = constraints.measure_text(label, &text).size.width;
                            Size::new(width, 0.0)
                        },
                    )
                    .style(style().min_width(0).height(0))
                    .hit_testable(false);
                column((
                    reservation,
                    text(label).style(crate::helpers::label_style().text_align(TextAlign::Center)),
                ))
                .style(style().min_width(0).align_items(Align::Stretch))
            };
            match hidden {
                Some(hidden) => show(move || !hidden.get(), visible).into_view(),
                None => visible().into_view(),
            }
        },
    )
    .into_view()
}

/// A toggle_button button named and labelled `label` that edits `pressed`.
///
/// It has `Role::Button` with a checked state (ARIA's `aria-pressed`); Space,
/// Enter and click flip it, and so do the accessibility `Activate` and
/// `ToggleButton` actions. The look is the theme's
/// [`ToggleButtonTheme::root`](crate::theme::ToggleButtonTheme), given a
/// [`ToggleButtonState`](crate::theme::ToggleButtonState). A toggle_button with an
/// `icon` and a hidden label is an icon toggle_button, a square: the recipe
/// knows it from [`ToggleButtonState::is_icon_only`].
///
/// ```no_run
/// use kivact::view::icon;
/// use kivact::prelude::*;
/// use kivact::ui::toggle_button;
///
/// fn bold() -> impl IntoView {
///     let on = signal(false);
///     toggle_button("Bold", on).icon(icon("bold")).hide_label(true)
/// }
/// ```
#[component]
pub fn toggle_button(
    label: Value<String>,
    pressed: Signal<bool>,
    on_edit: Option<Callback<EditPhase>>,
    icon: Option<View>,
    size: Option<Value<ControlSize>>,
    hide_label: Option<Value<bool>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let mut behavior = use_toggle_button(pressed);
    if let Some(observer) = on_edit {
        behavior = behavior.on_edit(move |phase| observer.call(phase));
    }
    // `.disabled(..)` ignores input while true.
    if let Some(disabled) = caller.disabled() {
        behavior = behavior.disabled(move || disabled.get());
    }
    let size = size.unwrap_or_else(|| default_control_size().into_value());
    let leading_icon = icon.is_some();
    let part = behavior
        .control()
        .map_state(move |state: ToggleState| ToggleButtonState {
            input: state.input,
            pressed: state.checked.is_checked(),
            size: size.get(),
            leading_icon,
            label_hidden: hide_label.is_some_and(|hide| hide.get()),
        });
    let state = part.state();
    let root: Node = node()
        .name(label)
        .base_style(crate::helpers::single_line_text());
    with_base_recipe_style(
        root,
        |r: &ToggleButtonTheme| r.root_recipe(),
        &variants,
        move || state.get(),
    )
    .part(part)
    .child(icon)
    .child(toggle_label(label, hide_label, size))
}
