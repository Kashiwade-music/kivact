//! Role: Timeline entries with plain or application-provided rich titles.

use crate::ColorName;
use crate::{Hue, Tokens};
use kivact::state::Value;
use kivact::state::{IntoText, IntoValue};
use kivact::style::use_theme;
use kivact::style::{Align, ColorToken, Direction, Justify, style};
use kivact::view::icon;
use kivact::view::{IntoView, View, show};
use kivact::view::{Node, NodeBuilder, column, node, row, text};

/// An activity entry. Created by [`timeline_item`].
pub struct TimelineItem {
    root: Node,
    title: Value<String>,
    title_content: Option<kivact::view::View>,
    detail: Value<String>,
    timestamp: Value<String>,
    glyph: Option<kivact::state::Value<kivact::style::Icon>>,
    hue: Value<Hue>,
}

/// An activity entry with a title, optional quotation and timestamp.
///
/// [`TimelineItem::title_content`] replaces the title text with rich text,
/// links or another public view.
#[track_caller]
pub fn timeline_item(title: impl IntoText) -> TimelineItem {
    TimelineItem {
        root: row(())
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
                        ColorToken::Border,
                    );
                }
            }),
        title: title.into_text_value(),
        title_content: None,
        detail: String::new().into_value(),
        timestamp: String::new().into_value(),
        glyph: None,
        hue: Hue::Gray.into_value(),
    }
}
impl TimelineItem {
    /// Supporting detail; empty by default.
    pub fn detail(mut self, value: impl IntoText) -> Self {
        self.detail = value.into_text_value();
        self
    }
    /// Timestamp; empty by default.
    pub fn timestamp(mut self, value: impl IntoText) -> Self {
        self.timestamp = value.into_text_value();
        self
    }
    /// Optional reactive icon.
    pub fn icon(mut self, value: impl IntoValue<kivact::style::Icon>) -> Self {
        self.glyph = Some(value.into_value());
        self
    }
    /// Icon hue; neutral gray by default.
    pub fn tone(mut self, value: impl IntoValue<Hue>) -> Self {
        self.hue = value.into_value();
        self
    }

    /// An application-owned title view in place of the title text, built
    /// once by `title` when the entry mounts.
    pub fn title_content(mut self, title: impl IntoView) -> Self {
        self.title_content = Some(title.into_view());
        self
    }
}
impl NodeBuilder for TimelineItem {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for TimelineItem {
    fn into_view(self) -> View {
        let Self {
            root,
            title,
            title_content,
            detail,
            timestamp,
            glyph,
            hue,
        } = self;
        let title = match title_content {
            Some(title) => title,
            None => text(title)
                .style(
                    style()
                        .font_size(14)
                        .line_height(1.5)
                        .weight(500)
                        .color(ColorToken::Text),
                )
                .into_view(),
        };
        root.child((
            node()
                .style(move || {
                    let t = &use_theme().get();
                    let p = t.ext::<Tokens>();
                    let (bg, fg) = p.hue(hue.get());
                    style()
                        .size(32, 32)
                        .shrink(0)
                        .radius(8)
                        .background(bg)
                        .color(fg)
                        .direction(Direction::Row)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                })
                .child(glyph.map(icon)),
            column((
                title,
                show(
                    move || !detail.get().is_empty(),
                    move || {
                        text(detail).style(
                            style()
                                .font_size(13)
                                .line_height(1.5)
                                .color(ColorToken::Text)
                                .margin_top(6)
                                .padding_x(12)
                                .padding_y(8)
                                .radius(8)
                                .background(ColorToken::SurfaceAlt)
                                .border(1, ColorToken::Border),
                        )
                    },
                ),
                text(timestamp)
                    .style(style().font_size(12).line_height(1.5))
                    .style(style().color(ColorName::TextFaint)),
            ))
            .style(
                style()
                    .padding_top(5)
                    .gap(2)
                    .grow(1)
                    .min_width(0)
                    .font_size(14)
                    .line_height(1.5)
                    .color(ColorToken::TextMuted),
            ),
        ))
        .into_view()
    }
}
