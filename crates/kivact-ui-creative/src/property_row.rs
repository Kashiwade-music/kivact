//! Role: Inspector property rows pairing a label with its value control.
use crate::PropertyState;
use crate::color::{ACCENT, ACCENT_HIGH, MUTED};
use kivact::component;
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::Point;
use kivact::i18n::message;
use kivact::state::{Value, memo};
use kivact::style::{Align, TextAlign, style};
use kivact::ui::{ButtonVariant, button, tooltip};
use kivact::view::{Element, Role, View, node, row, text};

/// A label with the control that edits its value, in one inspector row.
///
/// The row is a canonical description `Role::ListItem` named like the label.
/// Place related rows inside `kivact::ui::description_list` or a `Role::List`
/// container. The label is right aligned in a column of `label_width` pixels
/// (90 by default; a negative or non-finite width is 90) on one line, and
/// shows the full label on hover and focus in a tooltip (also when the label
/// is not clipped). `row_height` is the least height of the row in logical
/// pixels (at least 20). The value takes the rest of the row. Two slots hold
/// other controls: `leading` between the label and the value, such as a
/// [`property_state_button`], and `trailing` after the value, such as a
/// connection or an action button. The row owns no state; the controls it
/// holds do.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_creative as ui;
///
/// fn opacity() -> impl IntoView {
///     let opacity = signal(1.0);
///     ui::property_row("Opacity", ui::value_slider("Opacity", opacity).label_visible(false))
///         .leading(ui::property_state_button("Opacity", ui::PropertyState::Plain))
/// }
/// ```
#[component]
pub fn property_row(
    label: Value<String>,
    value: View,
    leading: Option<View>,
    trailing: Option<View>,
    #[prop(default = 90.0)] label_width: Value<f64>,
    #[prop(default = 24.0)] row_height: Value<f64>,
) -> impl Element {
    let label_width = memo(move || {
        let px = label_width.get();
        if px.is_finite() && px >= 0.0 {
            px
        } else {
            90.0
        }
    });
    let row_height = memo(move || {
        let px = row_height.get();
        if px.is_finite() { px.max(20.0) } else { 24.0 }
    });
    let name = tooltip(
        label,
        node()
            .style(move || style().width(label_width.get()).flex_shrink(0).min_width(0))
            .child(
                text(label)
                    .style(
                        style()
                            .font_size(11.5)
                            .text_align(TextAlign::End)
                            .single_line(),
                    )
                    .style(style().color(MUTED)),
            ),
    )
    .placement(kivact::view::Placement::Below)
    .align(kivact::view::OverlayAlign::Center)
    .offset(6.0);
    let detail = row((
        leading,
        node().style(style().flex_grow(1).min_width(0)).child(value),
        trailing,
    ))
    .style(
        style()
            .width(kivact::style::pct(100))
            .min_width(0)
            .gap(4)
            .align_items(Align::Center),
    );
    kivact::ui::description(label, detail)
        .term_content(name)
        .with_node(move |node| {
            node.role(Role::ListItem).style(move || {
                style()
                    .min_width(0)
                    .min_height(row_height.get())
                    .gap(4)
                    .align_items(Align::Center)
            })
        })
}

/// A small button that shows whether a property is animated, and lets the
/// application change it.
///
/// The glyph is an outlined square for [`PropertyState::Plain`], an outlined
/// diamond for `Animated`, a filled diamond for `Keyed` and a filled square
/// for `Driven`, in the Creative theme's accent colors. The button's name is
/// `label` and the state's name ("Opacity: Keyframe"). Hovering or focusing it
/// shows the state's name, from the messages `property.state.plain`,
/// `property.state.animated`, `property.state.keyed` and
/// `property.state.driven`. The state is only shown: what pressing the button
/// means, such as adding a keyframe, is up to `on_press`. `state` accepts a
/// signal, a closure or a plain [`PropertyState`] for a constant, and changes
/// the glyph without mounting anything again.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_creative as ui;
///
/// fn animate() -> impl IntoView {
///     let state = signal(ui::PropertyState::Plain);
///     ui::property_state_button("Opacity", state)
///         .on_press(move || state.set(ui::PropertyState::Keyed))
/// }
/// ```
#[component]
pub fn property_state_button(label: Value<String>, state: Value<PropertyState>) -> impl Element {
    let names = [
        message("creative.property.state.plain", "Not animated"),
        message("creative.property.state.animated", "Animated"),
        message("creative.property.state.keyed", "Keyframe"),
        message("creative.property.state.driven", "Driven"),
    ];
    let state_name = memo(move || {
        let index = match state.get() {
            PropertyState::Plain => 0,
            PropertyState::Animated => 1,
            PropertyState::Keyed => 2,
            PropertyState::Driven => 3,
        };
        names[index].get()
    });
    let name = memo(move || format!("{}: {}", label.get(), state_name.get()));
    let trigger = button("")
        .with_node(move |node| node.name(name))
        .variant(ButtonVariant::Ghost)
        .with_node(move |node| node.style(style().size(20, 20).padding(0).border_radius(3)))
        .draw(move |d, size| {
            let state = state.get();
            let color = match state {
                PropertyState::Plain => MUTED,
                PropertyState::Animated | PropertyState::Driven => ACCENT,
                PropertyState::Keyed => ACCENT_HIGH,
            };
            let (x, y) = (size.width * 0.5, size.height * 0.5);
            let path = match state {
                PropertyState::Animated | PropertyState::Keyed => Path::new()
                    .move_to(Point::new(x, y - 5.0))
                    .line_to(Point::new(x + 5.0, y))
                    .line_to(Point::new(x, y + 5.0))
                    .line_to(Point::new(x - 5.0, y))
                    .close(),
                PropertyState::Plain | PropertyState::Driven => Path::new()
                    .move_to(Point::new(x - 3.5, y - 3.5))
                    .line_to(Point::new(x + 3.5, y - 3.5))
                    .line_to(Point::new(x + 3.5, y + 3.5))
                    .line_to(Point::new(x - 3.5, y + 3.5))
                    .close(),
            };
            if matches!(state, PropertyState::Keyed | PropertyState::Driven) {
                d.fill_path(&path, color);
            } else {
                d.stroke_path(
                    &path,
                    &Stroke::new(1.2).cap(LineCap::Round).join(LineJoin::Round),
                    color,
                );
            }
        });
    tooltip(state_name, trigger)
        .placement(kivact::view::Placement::Below)
        .align(kivact::view::OverlayAlign::Center)
        .offset(6.0)
}
