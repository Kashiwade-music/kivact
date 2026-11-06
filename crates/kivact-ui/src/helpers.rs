//! Role: Share text styles used by standard control labels.

use kivact_core::style::{Style, TextOverflow, style};

/// The text style of a control that shows one line: the text does not wrap
/// and ends in an ellipsis where it does not fit. It is the first style
/// layer of the control's root (a recipe may override it) and reaches the
/// label by inheritance.
pub(super) fn single_line_text() -> Style {
    style()
        .text_wrap(false)
        .text_overflow(TextOverflow::Ellipsis)
}

/// The style of a control's label node: it may shrink below its text's
/// width in a row, so a narrow control cuts the label.
pub(super) fn label_style() -> Style {
    style().min_width(0)
}

/// A control's visible label, left out while `hidden` reads true (the
/// control keeps its accessible name either way).
pub(super) fn visible_label(
    label: kivact_core::state::Value<String>,
    hidden: Option<kivact_core::state::Value<bool>>,
) -> kivact_core::view::View {
    use kivact_core::view::IntoView;
    let label = move || kivact_core::view::text(label).style(label_style());
    match hidden {
        None => label().into_view(),
        Some(hidden) => kivact_core::view::show(move || !hidden.get(), label).into_view(),
    }
}

/// The icon of an item, drawn while it has one (the name resolved in the
/// theme's icon set).
pub(super) fn item_icon(
    icon: impl Fn() -> Option<kivact_core::style::Icon> + 'static,
) -> kivact_core::view::View {
    use kivact_core::view::IntoView;
    kivact_core::view::dynamic(icon, |icon| icon.map(kivact_core::view::icon)).into_view()
}

/// An item's label over its description, the description styled by
/// `description` (left out while the item has none).
pub(super) fn label_and_description(
    label: impl Fn() -> String + 'static,
    description: impl Fn() -> Option<String> + 'static,
    styled: impl Fn(kivact_core::view::Node) -> kivact_core::view::Node + 'static,
) -> kivact_core::view::Node {
    let description = kivact_core::state::memo(description);
    kivact_core::view::column((
        kivact_core::view::text(label).style(label_style()),
        kivact_core::view::show(
            move || description.get().is_some(),
            move || {
                styled(kivact_core::view::text(move || {
                    description.get().unwrap_or_default()
                }))
            },
        ),
    ))
    .style(style().grow(1).min_width(0))
}
