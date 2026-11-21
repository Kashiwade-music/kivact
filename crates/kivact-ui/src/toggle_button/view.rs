//! Role: The toggle button builder, its label and its rendering.

use crate::toggle_button::look::ToggleButtonTheme;
use crate::toggle_button::look::ToggleLabelReservation;

use crate::control_size::ControlSize;
use crate::layer::with_base_recipe_style;
use crate::toggle_button::look::ToggleButtonState;
use kivact_core::custom::{Constraints, primitive};
use kivact_core::draw::TextFormat;
use kivact_core::geometry::Size;
use kivact_core::state::Signal;
use kivact_core::state::Value;
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::style::{Align, TextAlign, style, use_theme};
use kivact_core::view::{Element, Node, column, node, text};
use kivact_core::view::{IntoView, View, dynamic, show};
use kivact_interaction::{ToggleState, use_toggle_button};

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

// xtask-allow(state): legacy: icon, hide_label
/// A button that stays pressed, created by [`toggle_button`]. Its behavior and
/// semantics are those of [`use_toggle_button`].
pub struct ToggleButton {
    root: Node,
    label: Value<String>,
    pressed: Signal<bool>,
    on_edit: Option<kivact_core::state::Callback<kivact_core::view::EditPhase>>,
    icon: Option<View>,
    hide_label: Option<Value<bool>>,
    size: Value<ControlSize>,
}

/// A toggle_button button named and labelled `label` that edits `pressed`.
///
/// It has `Role::Button` with a checked state (ARIA's `aria-pressed`); Space,
/// Enter and click flip it, and so do the accessibility `Activate` and
/// `ToggleButton` actions. The look is the theme's
/// [`ToggleButtonTheme::root`](crate::theme::ToggleButtonTheme), given a
/// [`ToggleButtonState`](crate::theme::ToggleButtonState). A toggle_button with an
/// [`icon`](ToggleButton::icon) and a hidden label is an icon toggle_button, a square.
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
#[track_caller]
pub fn toggle_button(
    label: impl IntoText,
    pressed: kivact_core::state::Signal<bool>,
) -> ToggleButton {
    let label = label.into_text_value();
    ToggleButton {
        root: node().name(label),
        label,
        pressed,
        icon: None,
        on_edit: None,
        hide_label: None,
        size: crate::control_size::default_control_size().into_value(),
    }
}

impl ToggleButton {
    /// Observe the beginning and commit of each value edit.
    pub fn on_edit<R: kivact_core::view::HandlerResult>(
        mut self,
        f: impl Fn(kivact_core::view::EditPhase) -> R + 'static,
    ) -> Self {
        self.on_edit = Some(kivact_core::state::callback(f));
        self
    }

    /// The size (default [`ControlSize::Medium`]), given to the recipe in
    /// its state; the recipe sizes the toggle_button by it.
    #[track_caller]
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }

    /// A view before the label; the recipe knows it from
    /// [`ToggleButtonState::leading_icon`].
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    /// Keep the accessible name while omitting the visible label (while
    /// true; constant or reactive). With an [`icon`](Self::icon) the toggle_button
    /// is an icon toggle_button: the recipe knows it from
    /// [`ToggleButtonState::is_icon_only`].
    #[track_caller]
    pub fn hide_label(mut self, hide: impl IntoValue<bool>) -> Self {
        self.hide_label = Some(hide.into_value());
        self
    }
}

impl Element for ToggleButton {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

#[diagnostic::do_not_recommend]
impl IntoView for ToggleButton {
    fn into_view(mut self) -> View {
        let variants = self.root.authored().variants();
        let mut behavior = use_toggle_button(self.pressed);
        if let Some(observer) = self.on_edit {
            behavior = behavior.on_edit(move |phase| observer.call(phase));
        }
        // `.disabled(..)` ignores input while true.
        if let Some(disabled) = self.root.accessible_disabled() {
            behavior = behavior.disabled(move || disabled.get());
        }
        let (size, hide) = (self.size, self.hide_label);
        let leading_icon = self.icon.is_some();
        let part = behavior
            .control()
            .map_state(move |state: ToggleState| ToggleButtonState {
                input: state.input,
                pressed: state.checked.is_checked(),
                size: size.get(),
                leading_icon,
                label_hidden: hide.is_some_and(|hide| hide.get()),
            });
        let state = part.state();
        let root = self.root.base_style(crate::helpers::single_line_text());
        with_base_recipe_style(
            root,
            |r: &ToggleButtonTheme| r.root_recipe(),
            &variants,
            move || state.get(),
        )
        .part(part)
        .child(self.icon)
        .child(toggle_label(self.label, self.hide_label, size))
        .into_view()
    }
}
