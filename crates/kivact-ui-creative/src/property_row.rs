//! Role: Inspector property rows pairing a label with its value control.
use crate::ColorName;

use kivact::ui::{Button, ButtonVariant, button, tooltip};

use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::Point;
use kivact::i18n::message;
use kivact::state::{IntoText, IntoValue};
use kivact::state::{Value, memo};
use kivact::style::{Align, TextAlign, style};
use kivact::view::{IntoView, Role, View};
use kivact::view::{Node, NodeBuilder, node, row, text};

use crate::PropertyState;

/// A property of an inspector: its label, its value control and the slots
/// around them. See [`property_row`].
pub struct PropertyRow {
    root: Node,
    label: Value<String>,
    value: View,
    leading: Option<View>,
    trailing: Option<View>,
    label_width: Value<f64>,
    row_height: Value<f64>,
}

/// A label with the control that edits its value, in one inspector row.
///
/// The row is a canonical description `Role::ListItem` named like the label.
/// Place related rows inside `kivact::ui::description_list` or a `Role::List`
/// container. The label is right aligned in a column of [`label_width`](PropertyRow::label_width) pixels
/// (90 by default) on one line, and shows the full label on hover and focus
/// in a tooltip (also when the label is not clipped). The value takes the rest
/// of the row. Two slots hold other controls:
/// [`leading`](PropertyRow::leading) between the label and the value, such as
/// a [`property_state_button`], and [`trailing`](PropertyRow::trailing) after
/// the value, such as a connection or an action button. The row owns no state;
/// the controls it holds do.
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
#[track_caller]
pub fn property_row(label: impl IntoText, value: impl IntoView) -> PropertyRow {
    PropertyRow {
        root: row(()),
        label: label.into_text_value(),
        value: value.into_view(),
        leading: None,
        trailing: None,
        label_width: 90.0.into_value(),
        row_height: 24.0.into_value(),
    }
}

impl PropertyRow {
    /// A control between the label and the value, such as the state button
    /// of a keyframe.
    pub fn leading(mut self, view: impl IntoView) -> Self {
        self.leading = Some(view.into_view());
        self
    }

    /// A control after the value, such as a connection or an action button.
    pub fn trailing(mut self, view: impl IntoView) -> Self {
        self.trailing = Some(view.into_view());
        self
    }

    /// The width of the label column in logical pixels. The default is 90.
    /// A negative or non-finite width keeps the earlier one.
    pub fn label_width(mut self, px: impl IntoValue<f64>) -> Self {
        let px = px.into_value();
        self.label_width = memo(move || {
            let px = px.get();
            if px.is_finite() && px >= 0.0 {
                px
            } else {
                90.0
            }
        });
        self
    }
    /// The least height of the row in logical pixels.
    pub fn row_height(mut self, px: impl IntoValue<f64>) -> Self {
        let px = px.into_value();
        self.row_height = memo(move || {
            let px = px.get();
            if px.is_finite() { px.max(20.0) } else { 24.0 }
        });
        self
    }
}

impl NodeBuilder for PropertyRow {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for PropertyRow {
    fn into_view(self) -> View {
        let label = self.label;
        let label_width = self.label_width;
        let row_height = self.row_height;
        let name = tooltip(label)
            .placement(kivact::view::Placement::Below)
            .align(kivact::view::OverlayAlign::Center)
            .offset(6.0)
            .attach(
                node()
                    .style(move || style().width(label_width.get()).shrink(0).min_width(0))
                    .child(
                        text(label)
                            .style(
                                style()
                                    .font_size(11.5)
                                    .text_align(TextAlign::End)
                                    .single_line(),
                            )
                            .style(style().color(ColorName::Muted)),
                    ),
            );
        let detail = row((
            self.leading,
            node().style(style().grow(1).min_width(0)).child(self.value),
            self.trailing,
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
            .with_node(move |_| {
                self.root.role(Role::ListItem).style(move || {
                    style()
                        .min_width(0)
                        .min_height(row_height.get())
                        .gap(4)
                        .align_items(Align::Center)
                })
            })
            .into_view()
    }
}

/// A small button that shows whether a property is animated, and lets the
/// application change it. See [`property_state_button`].
pub struct PropertyStateButton {
    button: Button,
    hint: Value<String>,
}

/// The animation state of a property as a 20 by 20 pixel button with a glyph.
///
/// The glyph is an outlined square for [`PropertyState::Plain`], an outlined
/// diamond for `Animated`, a filled diamond for `Keyed` and a filled square
/// for `Driven`, in the Creative theme's accent colors. The button's name is
/// `label` and the state's name ("Opacity: Keyframe"). Hovering or focusing it
/// shows the state's name, from the messages `property.state.plain`,
/// `property.state.animated`, `property.state.keyed` and
/// `property.state.driven`. The state is only shown: what pressing the button
/// means, such as adding a keyframe, is up to
/// [`on_press`](PropertyStateButton::on_press). `state` accepts a signal, a closure or
/// a plain [`PropertyState`] for a constant, and changes the glyph without mounting anything again.
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
#[track_caller]
pub fn property_state_button(
    label: impl IntoText,
    state: impl IntoValue<PropertyState>,
) -> PropertyStateButton {
    let label = label.into_text_value();
    let state = state.into_value();
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
    let button = button("")
        .with_node(move |node| node.name(name))
        .variant(ButtonVariant::Ghost)
        .with_node(move |node| node.style(style().size(20, 20).padding(0).radius(3)))
        .draw(move |d, size| {
            let p = d.theme().ext::<crate::tokens::Tokens>();
            let state = state.get();
            let color = match state {
                PropertyState::Plain => p.muted,
                PropertyState::Animated | PropertyState::Driven => p.accent,
                PropertyState::Keyed => p.accent_high,
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
    PropertyStateButton {
        button,
        hint: state_name,
    }
}

impl PropertyStateButton {
    /// Called by a click, Enter, Space or an assistive technology's Activate.
    #[track_caller]
    pub fn on_press(mut self, f: impl Into<kivact::state::Callback<()>>) -> Self {
        self.button = self.button.on_press(f);
        self
    }
}

impl NodeBuilder for PropertyStateButton {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        self.button.node_mut()
    }
}

impl IntoView for PropertyStateButton {
    fn into_view(self) -> View {
        tooltip(self.hint)
            .placement(kivact::view::Placement::Below)
            .align(kivact::view::OverlayAlign::Center)
            .offset(6.0)
            .attach(self.button)
    }
}
