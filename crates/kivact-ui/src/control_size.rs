//! Role: The standard controls size and its inherited default.

/// The metrics a control is drawn at; the recipe decides what each size
/// means.
///
/// Buttons, text inputs and areas, selects, checkboxes, switches, radio
/// groups and sliders take one with `.size(..)` and pass it to their
/// recipes in their state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ControlSize {
    /// The most compact size, for dense toolbars and inline actions.
    ExtraSmall,
    /// A compact size.
    Small,
    /// The ordinary size.
    #[default]
    Medium,
    /// A roomy size.
    Large,
}

#[derive(Clone, Copy)]
struct InheritedControlSize(ControlSize);

/// Set the default size of standard controls created in `content`.
/// A control's explicit `.size(..)` wins, including `ControlSize::Medium`.
/// Nested providers use the nearest default.
pub fn provide_control_size<V: kivact_core::view::IntoView>(
    size: ControlSize,
    content: impl FnOnce() -> V + 'static,
) -> kivact_core::view::View {
    kivact_core::state::provide(InheritedControlSize(size), content)
}

pub(crate) fn default_control_size() -> ControlSize {
    kivact_core::state::try_use_context::<InheritedControlSize>()
        .map_or_else(ControlSize::default, |size| size.get().0)
}

kivact_core::state::constant_value!(ControlSize);
