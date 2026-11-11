//! Role: Render domain-specific conversation-preview rows with caller-owned state.

use crate::Hue;
use kivact::state::Value;
use kivact::view::{Element, Node, column, row, text};

use kivact::state::{IntoText, IntoValue};
use kivact::style::{Align, ColorToken, TextOverflow, style};
use kivact::view::{IntoView, Role, View, show};
/// A conversation row. Created by [`thread`].
pub struct Thread {
    root: Node,
    initials: Value<String>,
    portrait: Option<kivact::view::View>,
    title: Value<String>,
    preview: Value<String>,
    unread: Value<usize>,
}

/// A conversation row with selection and unread count supplied by the application.
///
/// [`Thread::portrait`] replaces the initials avatar with application-provided
/// avatar content.
#[track_caller]
pub fn thread(initials: impl IntoText, title: impl IntoText) -> Thread {
    let title = title.into_text_value();
    Thread {
        root: row(())
            .with_node(|root| root.role(Role::Button))
            .focusable(true)
            .name(title)
            .selected(false)
            .style(
                style()
                    .font_size(14)
                    .line_height(1.5)
                    .padding(8)
                    .gap(10)
                    .align_items(Align::Center)
                    .radius(8)
                    .hover(style().background(ColorToken::SurfaceAlt))
                    .selected(
                        style()
                            .background(ColorToken::Surface)
                            .shadow(kivact::draw::Shadow::new(0, 1, 3, ColorToken::Shadow)),
                    ),
            ),
        initials: initials.into_text_value(),
        portrait: None,
        title,
        preview: String::new().into_value(),
        unread: 0usize.into_value(),
    }
}
impl Thread {
    /// Preview text; empty by default.
    pub fn preview(mut self, value: impl IntoText) -> Self {
        self.preview = value.into_text_value();
        self
    }
    /// Reactive unread count; zero by default.
    pub fn unread(mut self, value: impl IntoValue<usize>) -> Self {
        self.unread = value.into_value();
        self
    }
    /// Reactive selection; false by default.
    pub fn selected(mut self, value: impl IntoValue<bool>) -> Self {
        self.root = self.root.selected(value);
        self
    }

    /// An application-provided avatar view in place of the initials avatar,
    /// built once by `portrait` when the row mounts.
    pub fn portrait(mut self, portrait: impl IntoView) -> Self {
        self.portrait = Some(portrait.into_view());
        self
    }
}
impl Element for Thread {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Thread {
    fn into_view(self) -> View {
        let Self {
            root,
            initials,
            portrait,
            title,
            preview,
            unread,
        } = self;
        let portrait = match portrait {
            Some(portrait) => portrait,
            None => {
                let initials = kivact::state::IntoText::into_text_value(initials);
                let look = crate::AvatarLook::default().size(32);
                kivact::ui::avatar(initials)
                    .fallback_content(move || kivact::view::text(initials))
                    .variant(look)
                    .status_content(look.decorations())
            }
            .into_view(),
        };
        root.child((
            portrait,
            column((
                text(title).style(style().font_size(14).weight(500).line_height(1.5)),
                text(preview).style(
                    style()
                        .font_size(12)
                        .line_height(1.5)
                        .color(ColorToken::TextMuted)
                        .text_wrap(false)
                        .text_overflow(TextOverflow::Ellipsis),
                ),
            ))
            .style(style().grow(1).min_width(0)),
            show(
                move || unread.get() > 0,
                move || {
                    kivact::ui::badge(move || unread.get().to_string())
                        .variant(crate::BadgeHue::new(Hue::Pink))
                        .variant(crate::BadgeVariant::Count)
                        .into_view()
                },
            ),
        ))
        .into_view()
    }
}
