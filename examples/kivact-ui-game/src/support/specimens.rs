//! Role: The specimen frame, the token swatch and the shape tiles of the game catalog.
use kivact::component;
use kivact::draw::{Layer, Path};
use kivact::geometry::Point;
use kivact::state::Value;
use kivact::state::{IntoText, IntoValue};
use kivact::style::{Align, BorderStyle, Justify, color, pct, style};
use kivact::style::{Shape, Transform};
use kivact::view::{Element, Node, View, column, node, row, text};
use kivact::view::{IntoView, Role};
use kivact_ui_game as g;
use kivact_ui_game::{cut_corner, hud, mono};
/// Supported prototype shape motifs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShapeKind {
    Cut,
    Skew,
    Bracket,
    Stripe,
}

/// A catalog specimen frame: its id and title in a header with a machine
/// caption on the right, its content and an optional note under it.
#[component]
pub(crate) fn specimen_frame(
    id: Value<String>,
    title: Value<String>,
    content: View,
    #[prop(default = String::new())] caption: Value<String>,
    note: Option<View>,
) -> impl Element {
    hud(column((
        row((
            mono(id).style(style().color(color::ACCENT)),
            text(title).style(style().font_weight(700).font_size(12.5)),
            node().style(style().flex_grow(1)),
            mono(caption).style(
                style()
                    .font_size(10.5)
                    .font_weight(500)
                    .color(g::color::TEXT_3),
            ),
        ))
        .style(
            style()
                .height(36)
                .padding_x(14)
                .gap(10)
                .align_items(Align::Center)
                .background(color::SURFACE_ALT)
                .border_bottom(1, color::BORDER),
        ),
        node().style(move || style().padding(20)).child(content),
        note.map(|note| {
            node().child(note).style(move || {
                style()
                    .padding_x(14)
                    .padding_y(10)
                    .font_size(11.5)
                    .color(g::color::TEXT_3)
                    .border_top(1, color::BORDER)
                    .border_style(BorderStyle::Dashed)
            })
        }),
    )))
}

/// Token swatch; activation copies the explicit hexadecimal label.
#[track_caller]
pub(crate) fn swatch(
    name: impl IntoText,
    hex: impl IntoText,
    color: impl IntoValue<kivact::style::Color>,
) -> Node {
    let hex = hex.into_text_value();
    let color = color.into_value();
    column((
        node().style(move || {
            style().height(54).background(color.get()).clip_path(
                Shape::polygon()
                    .point(0, 0)
                    .point(pct(100), 0)
                    .point(pct(100), pct(100) - 12)
                    .point(pct(100) - 12, pct(100))
                    .point(0, pct(100)),
            )
        }),
        column((
            text(name).style(style().font_size(12).font_weight(700)),
            mono(hex).style(style().color(color::TEXT_MUTED)),
        ))
        .style(style().padding_x(10).padding_y(10).gap(4)),
    ))
    .role(Role::Button)
    .name(hex)
    .focusable(true)
    .on_press(move || {
        kivact::task::spawn(kivact::platform::copy_text(hex.get()));
    })
    .style(
        style()
            .border(1, color::BORDER)
            .background(color::SURFACE)
            .hover(style().translate(0, -2).border_color(color::ACCENT))
            .padding(1),
    )
}

/// Shape specimen, also useful as a small application callout.
#[track_caller]
pub(crate) fn shape_tile(kind: ShapeKind, label: impl IntoText) -> View {
    let label = mono(label);
    let b = node().style(
        style()
            .size(92, 64)
            .align_items(Align::Center)
            .justify_content(Justify::Center),
    );
    match kind {
        ShapeKind::Cut => b
            .style(
                style()
                    .clip_path(cut_corner(14.))
                    .background(color::SELECTION)
                    .color(color::ACCENT),
            )
            .child(label.style(style().color(color::ACCENT)))
            .into_view(),
        ShapeKind::Skew => b
            .style(
                style()
                    .transform(Transform::new().skew_x(-12))
                    .background(color::ACCENT)
                    .color(color::ACCENT_TEXT),
            )
            .child(
                label.style(
                    style()
                        .transform(Transform::new().skew_x(12))
                        .color(color::ACCENT_TEXT),
                ),
            )
            .into_view(),
        ShapeKind::Bracket => hud(label)
            .style(
                style()
                    .size(92, 64)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center),
            )
            .into_view(),
        ShapeKind::Stripe => b
            .draw(|d, s| {
                d.layer(Layer::new().clip_rect(s.to_rect()), |d| {
                    d.fill_rect(s.to_rect(), g::color::NAVY);
                    let mut x = -s.height;
                    while x < s.width {
                        d.fill_path(
                            &Path::new()
                                .move_to(Point::new(x, s.height))
                                .line_to(Point::new(x + s.height, 0.))
                                .line_to(Point::new(x + s.height + 8., 0.))
                                .line_to(Point::new(x + 8., s.height))
                                .close(),
                            g::color::YELLOW,
                        );
                        x += 16.;
                    }
                });
            })
            .into_view(),
    }
}
