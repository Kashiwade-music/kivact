//! Role: Catalog specimen framing and token demonstration tiles.
use kivact::draw::{Layer, Path, Shape, Transform};
use kivact::geometry::Point;
use kivact::state::Value;
use kivact::state::{IntoText, IntoValue};
use kivact::style::{Align, BorderStyle, ColorToken, Justify, pct, style, use_theme};
use kivact::view::{IntoView, Role, View};
use kivact::view::{Node, NodeBuilder, column, node, row, text};
use kivact_ui_game::{Color, Tokens, cut_corner, hud, mono};
/// Supported prototype shape motifs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShapeKind {
    Cut,
    Skew,
    Bracket,
    Stripe,
}

/// A catalog specimen frame. Created by [`specimen`].
pub(super) struct Specimen<C: kivact::view::Component> {
    root: C,
    id: Value<String>,
    title: Value<String>,
    caption: Value<String>,
    content: View,
    note: Option<kivact::view::View>,
}

/// A catalog specimen frame, usable for diagnostics and documentation apps.
///
/// [`Specimen::caption`] adds a right-aligned machine caption to the header.
#[track_caller]
pub(super) fn specimen(
    id: impl IntoText,
    title: impl IntoText,
    content: impl IntoView,
) -> Specimen<impl kivact::view::Component> {
    Specimen {
        root: hud(()),
        id: id.into_text_value(),
        title: title.into_text_value(),
        caption: String::new().into_value(),
        content: content.into_view(),
        note: None,
    }
}
impl<C: kivact::view::Component> Specimen<C> {
    /// A right-aligned machine caption in the header; empty by default.
    pub(super) fn caption(mut self, caption: impl IntoText) -> Self {
        self.caption = caption.into_text_value();
        self
    }
    /// A full-width note under the content, set off by a dashed separator;
    /// mounted once with the specimen.
    pub(super) fn note(mut self, note: impl IntoView) -> Self {
        self.note = Some(note.into_view());
        self
    }
}
impl<C: kivact::view::Component> NodeBuilder for Specimen<C> {
    fn node_mut(&mut self) -> &mut Node {
        self.root.node_mut()
    }
}
impl<C: kivact::view::Component> IntoView for Specimen<C> {
    fn into_view(self) -> View {
        self.root
            .with_node(move |root| {
                root.child(column((
                    row((
                        mono(self.id).style(style().color(ColorToken::Accent)),
                        text(self.title).style(style().weight(700).font_size(12.5)),
                        node().style(style().grow(1)),
                        mono(self.caption)
                            .style(style().font_size(10.5).weight(500).color(Color::Text3)),
                    ))
                    .style(
                        style()
                            .height(36)
                            .padding_x(14)
                            .gap(10)
                            .align_items(Align::Center)
                            .background(ColorToken::SurfaceAlt)
                            .border_bottom(1, ColorToken::Border),
                    ),
                    node()
                        .style(move || {
                            let t = &use_theme().get();
                            style().padding(t.ext::<Tokens>().padding)
                        })
                        .child(self.content),
                    self.note.map(|note| {
                        node().child(note).style(move || {
                            let t = &use_theme().get();
                            style()
                                .padding_x(14)
                                .padding_y(10)
                                .font_size(11.5)
                                .color(t.ext::<Tokens>().text_3)
                                .border_top(1, t.colors().border)
                                .border_style(BorderStyle::Dashed)
                        })
                    }),
                )))
            })
            .into_view()
    }
}

/// Token swatch; activation copies the explicit hexadecimal label.
#[track_caller]
pub(super) fn swatch(
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
            text(name).style(style().font_size(12).weight(700)),
            mono(hex).style(style().color(ColorToken::TextMuted)),
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
            .border(1, ColorToken::Border)
            .background(ColorToken::Surface)
            .hover(style().translate(0, -2).border_color(ColorToken::Accent))
            .padding(1),
    )
}

/// Shape specimen, also useful as a small application callout.
#[track_caller]
pub(super) fn shape_tile(kind: ShapeKind, label: impl IntoText) -> View {
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
                    .background(ColorToken::Selection)
                    .color(ColorToken::Accent),
            )
            .child(label.style(style().color(ColorToken::Accent)))
            .into_view(),
        ShapeKind::Skew => b
            .style(
                style()
                    .transform(Transform::new().skew_x(-12))
                    .background(ColorToken::Accent)
                    .color(ColorToken::AccentText),
            )
            .child(
                label.style(
                    style()
                        .transform(Transform::new().skew_x(12))
                        .color(ColorToken::AccentText),
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
                    let p = d.theme().ext::<Tokens>();
                    d.fill_rect(s.to_rect(), p.navy);
                    let mut x = -s.height;
                    while x < s.width {
                        d.fill_path(
                            &Path::new()
                                .move_to(Point::new(x, s.height))
                                .line_to(Point::new(x + s.height, 0.))
                                .line_to(Point::new(x + s.height + 8., 0.))
                                .line_to(Point::new(x + 8., s.height))
                                .close(),
                            p.yellow,
                        );
                        x += 16.;
                    }
                });
            })
            .into_view(),
    }
}
