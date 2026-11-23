//! Role: Timeline entries with plain or application-provided rich titles.

use crate::Hue;
use crate::color;
use kivact::component;
use kivact::state::Value;
use kivact::style::color as core;
use kivact::style::{Align, FlexDirection, Icon, Justify, style};
use kivact::view::{View, column, node, row, show, text};

/// An activity entry with a title, optional detail and timestamp, and an
/// optional icon in the entry's hue.
///
/// `title_content` replaces the title text with rich text, links or another
/// public view.
#[component]
pub fn timeline_item(
    title: Value<String>,
    #[prop(default = "")] detail: Value<String>,
    #[prop(default = "")] timestamp: Value<String>,
    icon: Option<Value<Icon>>,
    #[prop(default = Hue::Gray)] tone: Value<Hue>,
    title_content: Option<View>,
) -> impl Element {
    let title = match title_content {
        Some(title) => title,
        None => text(title)
            .style(
                style()
                    .font_size(14)
                    .line_height(1.5)
                    .font_weight(500)
                    .color(core::TEXT),
            )
            .into_view(),
    };
    row(())
        .style(
            style()
                .gap(12)
                .padding_bottom(18)
                .font_size(14)
                .line_height(1.5),
        )
        .draw(|draw, size| {
            if size.height > 40.0 {
                draw.line(
                    kivact::geometry::Point::new(15.0, 36.0),
                    kivact::geometry::Point::new(15.0, size.height - 4.0),
                    1.0,
                    core::BORDER,
                );
            }
        })
        .child((
            node()
                .style(move || {
                    let (bg, fg) = tone.get().colors();
                    style()
                        .size(32, 32)
                        .flex_shrink(0)
                        .border_radius(8)
                        .background(bg)
                        .color(fg)
                        .flex_direction(FlexDirection::Row)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                })
                .child(icon.map(kivact::view::icon)),
            column((
                title,
                show(
                    move || !detail.get().is_empty(),
                    move || {
                        text(detail).style(
                            style()
                                .font_size(13)
                                .line_height(1.5)
                                .color(core::TEXT)
                                .margin_top(6)
                                .padding_x(12)
                                .padding_y(8)
                                .border_radius(8)
                                .background(core::SURFACE_ALT)
                                .border(1, core::BORDER),
                        )
                    },
                ),
                text(timestamp)
                    .style(style().font_size(12).line_height(1.5))
                    .style(style().color(color::TEXT_FAINT)),
            ))
            .style(
                style()
                    .padding_top(5)
                    .gap(2)
                    .flex_grow(1)
                    .min_width(0)
                    .font_size(14)
                    .line_height(1.5)
                    .color(core::TEXT_MUTED),
            ),
        ))
}
