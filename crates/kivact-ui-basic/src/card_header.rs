//! Role: The Lyra card header content: a title, a description and an action.
use crate::Tokens;
use kivact::state::{IntoText, Value};
use kivact::style::{Align, Direction, style, use_tokens};
use kivact::view::{Element, IntoView, Node, View, column, node, row, text};

/// The header of a standard `kivact::ui::card`: a title, a description and
/// an action. Created by [`card_header`]; pass it to the card's `.header(..)`.
/// The card's header region draws the spacing and, with
/// [`CardLook::header_bordered`](crate::CardLook::header_bordered), the rule.
pub struct CardHeader {
    root: Node,
    title: Value<String>,
    description: Option<Value<String>>,
    action: Option<View>,
}

/// A card header titled `title`.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn login() -> impl IntoView {
///     kivact::ui::card(text_input("Email", signal(String::new())))
///         .header(
///             basic::card_header("Login to your account")
///                 .description("Enter your email below to login")
///                 .action(button("Sign Up")),
///         )
///         .footer(button("Login"))
/// }
/// ```
#[track_caller]
pub fn card_header(title: impl IntoText) -> CardHeader {
    CardHeader {
        root: node(),
        title: title.into_text_value(),
        description: None,
        action: None,
    }
}

impl CardHeader {
    /// Show `text` under the title, in the muted color.
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(text.into_text_value());
        self
    }

    /// Show `action` at the end of the title, aligned to its top.
    pub fn action(mut self, action: impl IntoView) -> Self {
        self.action = Some(action.into_view());
        self
    }
}

impl Element for CardHeader {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for CardHeader {
    fn into_view(self) -> View {
        let titles = column((
            text(self.title).style(style().font_size(14).line_height(20.0 / 14.0).weight(500)),
            self.description.map(|description| {
                text(description).style(move || {
                    style()
                        .line_height(1.625)
                        .color(use_tokens::<Tokens>().muted_foreground)
                })
            }),
        ))
        .style(style().gap(4).grow(1).min_width(0));
        let action = self
            .action
            .map(|action| row(action).style(style().align_self(Align::Start).shrink(0)));
        self.root
            .base_style(
                style()
                    .direction(Direction::Row)
                    .align_items(Align::Start)
                    .gap(8)
                    .grow(1)
                    .min_width(0),
            )
            .child((titles, action))
            .into_view()
    }
}
