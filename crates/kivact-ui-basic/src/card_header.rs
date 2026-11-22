//! Role: The Lyra card header content: a title, a description and an action.

use crate::MUTED_FOREGROUND;
use kivact::component;
use kivact::custom::Caller;
use kivact::state::Value;
use kivact::style::{Align, FlexDirection, style};
use kivact::view::{View, column, node, row, text};

/// The header of a standard `kivact::ui::card`: a title, a description and
/// an action. Created by [`card_header`]; pass it to the card's `.header(..)`.
/// The card's header region draws the spacing and, with
/// [`CardHeaderBordered`](crate::CardHeaderBordered), the rule.
///
/// A card header titled `title`.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn login() -> impl IntoView {
///     kivact::ui::card(text_input(signal(String::new())).label("Email"))
///         .header(
///             basic::card_header("Login to your account")
///                 .description("Enter your email below to login")
///                 .action(button("Sign Up")),
///         )
///         .footer(button("Login"))
/// }
/// ```
#[component]
pub fn card_header(
    title: Value<String>,
    action: Option<View>,
    #[caller] caller: Caller,
) -> impl Element {
    let titles = column((
        text(title).style(
            style()
                .font_size(14)
                .line_height(20.0 / 14.0)
                .font_weight(500),
        ),
        caller.description().map(|description| {
            text(description).style(style().line_height(1.625).color(MUTED_FOREGROUND))
        }),
    ))
    .style(style().gap(4).flex_grow(1.0).min_width(0));
    let action =
        action.map(|action| row(action).style(style().align_self(Align::Start).flex_shrink(0.0)));
    node()
        .base_style(
            style()
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Start)
                .gap(8)
                .flex_grow(1.0)
                .min_width(0),
        )
        .child((titles, action))
}
